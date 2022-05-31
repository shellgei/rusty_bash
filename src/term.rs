//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::env;
use std::io::{Write, stdout, stdin};
use std::io::Stdout;

use termion::{event,terminal_size};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

use crate::ShellCore;
use crate::term_completion::*;

use crate::utils::chars_to_string;

extern crate unicode_width;
use unicode_width::UnicodeWidthStr;

pub struct Writer {
    pub stdout: RawTerminal<Stdout>, 
    pub chars: Vec<char>,
    pub fold_points: Vec<usize>,
    pub previous_fold_points_num: usize, 
    pub erased_line_num: usize,
    ch_ptr: usize,
    hist_ptr: usize,
    left_shift: u16,
}

fn char_to_width(c: char) -> u8{
    let s: &str = &c.to_string();
    UnicodeWidthStr::width(s) as u8
}

fn chars_to_width(chars: &Vec<char>) -> u32 {
    chars.iter()
        .map(|c| char_to_width(*c))
        .fold(0, |line_len, w| line_len + (w as u32))
}

impl Writer {
    pub fn new(hist_size: usize, left_shift: u16) -> Writer{
        Writer {
            stdout: stdout().into_raw_mode().unwrap(),
            chars: vec!(),
            fold_points: vec!(),
            previous_fold_points_num: 0,
            erased_line_num: 0,
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

    pub fn ch_ptr_to_multiline_origin(&mut self) -> (usize, u16) { 
        let mut y = 0;
        let mut x_from = 0;
        for p in &self.fold_points {
            if self.ch_ptr <= *p {
                break;
            }
            x_from = *p;
            y += 1;
        };

        (x_from, y)
    }

    pub fn terminal_size(&mut self) -> (u32, u32) {
        if let Ok((wx, wy)) = terminal_size(){
            (wx as u32, wy as u32)
        }else{
            panic!("Cannot get terminal size");
        }
    }

    pub fn write_history(&mut self, inc: i32, history: &Vec<String>){
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
        self.rewrite_line(y, h.to_string());
        self.chars.clear();
        self.chars = h.chars().collect();
        
        self.ch_ptr = self.chars.len();
        self.calculate_fold_points();
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
        let (_, old_line_no) = self.ch_ptr_to_multiline_origin();
        self.move_char_ptr(inc);
        let (org_x, line_no) = self.ch_ptr_to_multiline_origin();
        let line_len: u16 = chars_to_width(&self.chars[org_x..self.ch_ptr].to_vec()) as u16;

        let x = if line_no == 0{
            self.left_shift+line_len+1
        }else{
            line_len+1
        };

        let y = if old_line_no == line_no{
            self.cursor_pos().1
        }else{
            self.cursor_pos().1 + line_no - old_line_no
        };

        write!(self.stdout, "{}",
               termion::cursor::Goto(x, y)
               ).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn last_arg(&self) -> String {
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

        chars_to_string(&self.chars[pos..].to_vec())
    }

    fn calculate_fold_points(&mut self){
        let (wx, _) = self.terminal_size();
        self.previous_fold_points_num = self.fold_points.len();
        self.fold_points.clear();

        let mut i: usize = 0;
        let mut sum_length: u32 = 0;
        let mut shift = self.left_shift;
        for ch in &self.chars {
            sum_length += char_to_width(*ch) as u32;

            if wx < sum_length + shift as u32 {
                shift = 0;
                sum_length = char_to_width(*ch) as u32;
                self.fold_points.push(i);
            }
            i += 1;
        }
    }

    fn tab_completion(&mut self, tab_num: u32, core: &mut ShellCore) {
        if chars_to_string(&self.chars) == self.last_arg() {
            if tab_num == 1 {
                command_completion(self);
            }else if tab_num == 2 {
                show_command_candidates(self, core);
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

    fn write_multi_line(&mut self, y: u16, org_y: u16) {
        write!(self.stdout, "{}{}", 
               termion::cursor::Goto(self.left_shift , y - org_y),
               termion::clear::UntilNewline,
        ).unwrap();

        let mut clear_y: u16 = y - org_y + 1;
        let (_, wy) = self.terminal_size();
        while clear_y <= wy as u16 {
            write!(self.stdout, "{}{}", 
                   termion::cursor::Goto(0 , clear_y),
                   termion::clear::UntilNewline,
            ).unwrap();
            clear_y += 1;
        }

        self.rewrite_line(y - org_y, self.chars.iter().collect());
        //write!(self.stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
        //self.stdout.flush().unwrap();
    }

    fn remove(&mut self) {
        if self.chars.len() == 0 {
            return;
        };

        let (_, old_org_y) = self.ch_ptr_to_multiline_origin();
        self.move_char_ptr(-1);

        self.chars.remove(self.ch_ptr);

        let (org_x, org_y) = self.ch_ptr_to_multiline_origin();
        let line_len: u16 = chars_to_width(&self.chars[org_x..self.ch_ptr].to_vec()) as u16;

        let x = if org_y == 0{
            self.left_shift+line_len+1
        }else{
            line_len+1
        };

        let y = if old_org_y == org_y{
            self.cursor_pos().1
        }else{
            self.cursor_pos().1 + org_y - old_org_y
        };

        self.write_multi_line(y, org_y);
        write!(self.stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn insert(&mut self, c: char) {
        let (_, old_org_y) = self.ch_ptr_to_multiline_origin();

        self.chars.insert(self.ch_ptr, c);
        self.move_char_ptr(1);
        self.calculate_fold_points();

        let (org_x, org_y) = self.ch_ptr_to_multiline_origin();
        let line_len: u16 = chars_to_width(&self.chars[org_x..self.ch_ptr].to_vec()) as u16;

        let x = if org_y == 0{
            self.left_shift+line_len+1
        }else{
            line_len+1
        };

        let y = if old_org_y == org_y{
            self.cursor_pos().1
        }else{
            self.cursor_pos().1 + org_y - old_org_y
        };

        self.write_multi_line(y, org_y);
        write!(self.stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
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

    let ans = chars_to_string(&writer.chars);//writer.chars.iter().collect::<String>();
    core.history.push(ans.clone());
    ans + "\n"
}
