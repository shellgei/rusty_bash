//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::env;
use std::collections::HashSet;
use std::io::{Write, stdout, stdin};
use std::io::Stdout;

use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

use crate::core::History;
use crate::ShellCore;
use crate::utils::{eval_glob, search_commands};


fn compare_nth_char(nth: usize, strs: &Vec<String>) -> bool {
    if strs.len() < 2 {
        return false;
    };

    let ch0: char;
    if let Some(ch) = &strs[0].chars().nth(nth){
        ch0 = *ch;
    }else{
        return false;
    }

    for s in strs {
        if let Some(ch) = s.chars().nth(nth){
            if ch != ch0{
                return false;
            }
        }else{
            return false;
        }
    }

    true
}


fn file_completion(writer: &mut Writer){
    let s: String = writer.last_arg() + "*";
    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    let base_len = writer.last_arg().len();
    if ans.len() == 1 {
        for ch in ans[0][base_len..].chars() {
            writer.insert(ch);
        }
    }else{
        for (i, ch) in ans[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &ans) {
                writer.insert(ch);
            }else{
                break;
            }
        }
    }
}


fn show_file_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s: String = writer.last_arg() + "*";
    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    write!(writer.stdout, "\r\n").unwrap();
    for f in ans {
        write!(writer.stdout, "{}        ", f).unwrap();
    }
    write!(writer.stdout, "\r\n").unwrap();
    writer.stdout.flush().unwrap();
    prompt(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
    return;
}

fn command_completion(writer: &mut Writer, tab_num: u32, core: &mut ShellCore) {
    let paths = search_commands(&(writer.chars.iter().collect::<String>() + "*"));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    let base_len = writer.last_arg().len();
    if keys.len() == 1 {
        for ch in keys[0][base_len..].chars() {
            writer.insert(ch);
        }
        return;
    }else{
        for (i, ch) in keys[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &keys) {
                writer.insert(ch);
            }else{
                break;
            }
        }
        return;
    };
}

fn show_command_candidates(writer: &mut Writer, tab_num: u32, core: &mut ShellCore) {
    let paths = search_commands(&(writer.chars.iter().collect::<String>() + "*"));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    write!(writer.stdout, "\r\n").unwrap();
    for f in keys {
        write!(writer.stdout, "{}        ", f).unwrap();
    }
    write!(writer.stdout, "\r\n").unwrap();
    writer.stdout.flush().unwrap();
    prompt(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
}

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

    fn last_arg(&self) -> String {
        let mut escaped = false;
        let mut pos = 0;
        let mut counter = 0;
        for ch in self.chars.clone() {
            if escaped{
                escaped = false;
            }else if ch == '\\' {
                escaped = true;
            }

            if !escaped && ch == ' '{
                pos = counter+1;
            }
            counter += 1;
        }

        self.chars[pos..].iter().collect::<String>()
    }


    fn tab_completion(&mut self, tab_num: u32, core: &mut ShellCore) {
        if tab_num > 2 {
            return;
        };

        if self.chars.iter().collect::<String>() == self.last_arg() {
            if tab_num == 1 {
                command_completion(self, tab_num, core);
            }else{
                show_command_candidates(self, tab_num, core);
            };
        }else{
            if tab_num == 1 {
                file_completion(self);
            }else if tab_num == 2 {
                show_file_candidates(self, core);
                return;
            };
        };
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

pub fn prompt(core: &mut ShellCore) -> u16 {
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

    let host = core.vars["HOSTNAME"].clone();

    print!("\x1b[33m\x1b[1m{}@{}\x1b[m\x1b[m:", user, host);
    print!("\x1b[35m\x1b[1m{}\x1b[m\x1b[m", path);
    print!("$ ");
    io::stdout().flush().unwrap();

    (user.len() + host.len() + path.len() + 2 + 2) as u16
}

pub fn read_line(left: u16, core: &mut ShellCore) -> String{
    let mut writer = Writer::new(core.history.len(), left);
    let mut tab_num = 0;

    for c in stdin().keys() {
        match &c.as_ref().unwrap() {
            event::Key::Ctrl('c') => {
                writer.chars.clear();
                writer.end("^C\r\n");
                break;
            },
            event::Key::Char('\n') => {
                writer.end("\r\n");
                break;
            },
            event::Key::Up         => writer.write_history(-1, &core.history),
            event::Key::Down       => writer.write_history(1, &core.history),
            event::Key::Left       => writer.move_cursor(-1),
            event::Key::Right      => writer.move_cursor(1),
            event::Key::Backspace  => writer.remove(),
            event::Key::Char('\t') => writer.tab_completion(tab_num+1, core),
            event::Key::Char(ch)    => writer.insert(*ch),
            _  => {},
        }

        if c.unwrap() != event::Key::Char('\t') {
            tab_num = 0;
        }else{
            tab_num += 1;
        }
    }

    let ans = writer.chars.iter().collect::<String>();
    core.history.push(History{commandline: ans.clone(), charwidths: writer.widths});
    ans + "\n"
}
