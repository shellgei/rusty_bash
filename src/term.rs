//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin};
use std::convert::TryInto;
use std::io::Stdout;

use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

use crate::core::History;


struct Writer {
    pub stdout: RawTerminal<Stdout>, 
    pub chars: Vec<char>,
    pub widths: Vec<u8>,
    pub ch_ptr: usize,
    pub hist_ptr: usize,
    pub left_shift: u16,
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

    pub fn write_history(&mut self, y: u16, inc: i32, history: &Vec<History>){
        if history.len() == 0 {
            return;
        }

        if self.hist_ptr as i32 + inc < 0 {
            self.hist_ptr = 0;
        }else{
            self.hist_ptr = (self.hist_ptr as i32 + inc) as usize;
        }

        if self.hist_ptr >= history.len() {
            self.hist_ptr = history.len();
            return;
        }

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

    fn move_cursor(&mut self, inc: i32, y: u16){
        self.move_char_ptr(inc);
        let line_len: u16 = self.widths[0..self.ch_ptr].iter().fold(0, |line_len, w| line_len + (*w as u16));
        write!(self.stdout, "{}", termion::cursor::Goto(self.left_shift+line_len+1, y)).unwrap();
        self.stdout.flush().unwrap();
    }
}

pub fn prompt(text: &String) -> u16 {
    let prompt = format!("{} $ ", text);
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    prompt.len().try_into().unwrap()
}


pub fn read_line(left: u16, history: &mut Vec<History>) -> String{
    let mut writer = Writer::new(history.len(), left);

    for c in stdin().keys() {
        let (x, y) = writer.cursor_pos();
        match c.unwrap() {
            event::Key::Ctrl('c') => {
                writer.chars.clear();
                write!(writer.stdout, "^C\n").unwrap();
                break;
            },
            event::Key::Up => writer.write_history(y, -1, &history),
            event::Key::Down => writer.write_history(y, 1, &history),
            event::Key::Left => writer.move_cursor(-1, y),
            event::Key::Right => writer.move_cursor(1, y),
            event::Key::Backspace => {
                if writer.chars.len() == 0 {
                    continue;
                };

                writer.move_char_ptr(-1);
                writer.chars.remove(writer.ch_ptr);
                let removed_width = writer.widths[writer.ch_ptr];
                writer.widths.remove(writer.ch_ptr);
                writer.rewrite_line(y, writer.chars.iter().collect());

                if x - removed_width as u16 >= left {
                    write!(writer.stdout, "{}", termion::cursor::Goto(x-removed_width as u16, y)).unwrap();
                    writer.stdout.flush().unwrap();
                }
            },
            event::Key::Char(c) => {
                    if c == '\n' {
                        write!(writer.stdout, "{}", c).unwrap();
                        writer.chars.push(c);
                        break;
                    }

                    if writer.ch_ptr <= writer.chars.len() {
                        writer.chars.insert(writer.ch_ptr, c);
                        writer.ch_ptr += 1;

                        /* output the line before the cursor */
                        writer.rewrite_line(y, writer.chars[0..writer.ch_ptr].iter().collect());
                        let (new_x, new_y) = writer.cursor_pos();
                        writer.widths.insert(writer.ch_ptr-1, (new_x - x) as u8);
    
                        /* output the line after the cursor */
                        write!(writer.stdout, "{}{}",
                               writer.chars[writer.ch_ptr..].iter().collect::<String>(), 
                               termion::cursor::Goto(new_x, new_y),
                        ).unwrap();
                    }else{
                        eprintln!("ch_ptr: {}, {}", writer.ch_ptr, writer.chars.len());
                    };

                    writer.stdout.flush().unwrap();
            },
            _ => {},
        }
    }

    write!(writer.stdout, "\r").unwrap();
    writer.stdout.flush().unwrap();
    let ans = writer.chars.iter().collect::<String>();

    history.push(History{
        commandline: ans[0..ans.len()-1].to_string(),
        charwidths: writer.widths});

    ans
}

