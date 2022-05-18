//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::env;
use std::io::{Write, stdout, stdin};
use std::io::Stdout;

use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

use crate::core::History;


struct Writer {
    stdout: RawTerminal<Stdout>, 
    chars: Vec<char>,
    widths: Vec<u8>,
    ch_ptr: usize,
    hist_ptr: usize,
    left_shift: u16,
}

impl Writer {
    pub fn new(hist_size: usize, left_shift: u16) -> Writer{
        Writer {
            stdout: stdout().into_raw_mode().unwrap(),
            chars: vec!(),
            widths: vec!(),
            ch_ptr: 0,
            hist_ptr: hist_size,
            left_shift: left_shift,
        }
    }

    pub fn rewrite_line(&mut self, y: u16, text: String){
        write!(self.stdout, "{}{}{}",
               termion::cursor::Goto(self.left_shift+1, y),
               termion::clear::UntilNewline,
               text).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn cursor_pos(&mut self) -> (u16, u16) {
        self.stdout.cursor_pos().unwrap()
    }

    pub fn write_history(&mut self, inc: i32, history: &Vec<History>){
        if history.len() == 0 {
            return;
        }

        self.hist_ptr = if self.hist_ptr as i32 + inc < 0 {
            0
        }else{
            (self.hist_ptr as i32 + inc) as usize
        };

        if self.hist_ptr >= history.len() {
            self.hist_ptr = history.len();
            return;
        }

        let y = self.cursor_pos().1;
        let h = &history[self.hist_ptr as usize];
        self.rewrite_line(y, h.commandline.to_string());
        self.chars.clear();
        self.widths.clear();
        self.chars = h.commandline.chars().collect();
        self.widths = h.charwidths.clone();
        self.ch_ptr = self.widths.len();

        if self.chars.len() != self.widths.len() {
            panic!("Broken history data: \n\r{:?}, \n\r{:?}\n\r", self.chars, self.widths);
        };
    }

    fn move_char_ptr(&mut self, inc: i32){
       let pos = self.ch_ptr as i32 + inc; 

       self.ch_ptr = if pos < 0 {
           0
       }else if pos > self.chars.len() as i32 {
           self.chars.len()
       }else{
           pos as usize
       }
    }

    fn move_cursor(&mut self, inc: i32) {
        self.move_char_ptr(inc);
        let line_len: u16 = self.widths[0..self.ch_ptr]
            .iter()
            .fold(0, |line_len, w| line_len + (*w as u16));

        let y = self.cursor_pos().1;
        write!(self.stdout, "{}",
               termion::cursor::Goto(self.left_shift+line_len+1, y)
               ).unwrap();
        self.stdout.flush().unwrap();
    }

    fn remove(&mut self) {
        let (x, y) = self.cursor_pos();
        if self.chars.len() == 0 {
            return;
        };

        self.move_char_ptr(-1);
        self.chars.remove(self.ch_ptr);
        let new_x = if x >= self.widths[self.ch_ptr] as u16 {
            x - self.widths[self.ch_ptr] as u16
        }else{
            self.left_shift
        };

        self.widths.remove(self.ch_ptr);
        self.rewrite_line(y, self.chars.iter().collect());
        write!(self.stdout, "{}", termion::cursor::Goto(new_x, y)).unwrap();
        self.stdout.flush().unwrap();
    }

    fn insert(&mut self, c: char) {
        let (x, y) = self.cursor_pos();
        if self.ch_ptr > self.chars.len() {
            return;
        };

        self.chars.insert(self.ch_ptr, c);
        self.ch_ptr += 1;

        /* output the line before the cursor */
        self.rewrite_line(y, self.chars[0..self.ch_ptr].iter().collect());
        let (new_x, new_y) = self.cursor_pos();
        self.widths.insert(self.ch_ptr-1, (new_x - x) as u8);

        /* output the line after the cursor */
        write!(self.stdout, "{}{}",
               self.chars[self.ch_ptr..].iter().collect::<String>(), 
               termion::cursor::Goto(new_x, new_y),
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn end(&mut self, text: &str) {
        write!(self.stdout, "{}", text).unwrap();
    }
}

pub fn prompt() -> u16 {
    let home = if let Ok(h) = env::var("HOME"){
        h
    }else{
        "unknown".to_string()
    };

    let path = if let Ok(p) = env::current_dir(){
        p.into_os_string()
            .into_string()
            .unwrap()
            .replace(&home, "~")
    }else{
        "no_path".to_string()
    };

    let user = if let Ok(u) = env::var("USER"){
        u
    }else{
        "unknown".to_string()
    };

    let host = if let Ok(h) = env::var("HOSTNAME"){
        h
    }else{
        "unknown".to_string()
    };

    print!("\x1b[33m\x1b[1m{}@{}\x1b[m\x1b[m ", user, host);
    print!("\x1b[35m\x1b[1m{}\x1b[m\x1b[m", path);
    print!("$ ");
    io::stdout().flush().unwrap();

    (user.len() + host.len() + path.len() + 10 + 2) as u16
}

pub fn read_line(left: u16, history: &mut Vec<History>) -> String{
    let mut writer = Writer::new(history.len(), left);

    for c in stdin().keys() {
        match c.unwrap() {
            event::Key::Ctrl('c') => {
                writer.chars.clear();
                writer.end("^C\r\n");
                break;
            },
            event::Key::Char('\n') => {
                writer.end("\r\n");
                break;
            },
            event::Key::Up        => writer.write_history(-1, &history),
            event::Key::Down      => writer.write_history(1, &history),
            event::Key::Left      => writer.move_cursor(-1),
            event::Key::Right     => writer.move_cursor(1),
            event::Key::Backspace => writer.remove(),
            event::Key::Char(c)   => writer.insert(c),
            _ => {},
        }
    }

    let ans = writer.chars.iter().collect::<String>();
    history.push(History{commandline: ans.clone(), charwidths: writer.widths});
    ans + "\n"
}
