//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin, Stdout};

use termion::{event,terminal_size};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;

use crate::ShellCore;

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
    hist_ptr: i32,
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
    pub fn new(hist_len: usize, left_shift: u16) -> Writer{
        Writer {
            stdout: stdout().into_raw_mode().unwrap(),
            chars: vec![],
            fold_points: vec![],
            previous_fold_points_num: 0,
            erased_line_num: 0,
            ch_ptr: 0,
            hist_ptr: hist_len as i32,
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
        if let Ok(x) = self.stdout.cursor_pos(){
            x
        }else{
            (0, 0)
        }
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

    pub fn call_history(&mut self, inc: i32, history: &Vec<String>){
        let len = history.len() as i32;
        if len == 0 {
            return;
        }
        self.hist_ptr += inc;

        let h = if self.hist_ptr < 0 {
            self.hist_ptr = 0;
            return;
        }else if self.hist_ptr < len {
            history[self.hist_ptr as usize].to_string()
        }else{
            self.hist_ptr = len;
            "".to_string()
        };

        let y = self.cursor_pos().1;
        self.rewrite_line(y, h.clone());
        self.chars.clear();
        self.chars = h.chars().collect();
        self.ch_ptr = self.chars.len();
        self.calculate_fold_points();
    }

    pub fn move_char_ptr(&mut self, inc: i32){
       let pos = self.ch_ptr as i32 + inc; 

       self.ch_ptr = if pos < 0 {
           0
       }else if pos > self.chars.len() as i32 {
           self.chars.len()
       }else{
           pos as usize
       }
    }

    fn move_cursor_to_head(&mut self) {
        let y = self.cursor_pos().1;
        let org_y = self.ch_ptr_to_multiline_origin().1;
        write!(self.stdout, "{}",
               termion::cursor::Goto(self.left_shift + 1, y - org_y)
            ).unwrap();
        self.stdout.flush().unwrap();
        self.ch_ptr = 0;
    }

    fn move_cursor_to_tail(&mut self) {
        let y = self.cursor_pos().1;
        let org_y = self.ch_ptr_to_multiline_origin().1;
        self.rewrite_line(y - org_y, chars_to_string(&self.chars));
        self.ch_ptr = self.chars.len();
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
    }

    fn rewrite_multi_line(&mut self, old_org_y: u16) {
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

    fn remove(&mut self) {
        if self.chars.len() == 0 {
            return;
        };

        let (_, old_org_y) = self.ch_ptr_to_multiline_origin();
        self.move_char_ptr(-1);
        self.chars.remove(self.ch_ptr);

        self.rewrite_multi_line(old_org_y);
        self.calculate_fold_points();
    }

    pub fn insert(&mut self, c: char) {
        if self.ch_ptr == self.chars.len() {
            self.chars.insert(self.ch_ptr, c);
            self.move_char_ptr(1);
            let _ = write!(self.stdout, "{}", c);
            self.stdout.flush().unwrap();
            self.calculate_fold_points();
            return;
        }else{
            let mut remain = self.chars[self.ch_ptr..].to_vec();
            self.chars = self.chars[0..self.ch_ptr].to_vec();

            remain.insert(0, c);
            self.chars.append(&mut remain.clone());
            self.move_char_ptr(remain.len() as i32);
            let _ = write!(self.stdout, "{}", chars_to_string(&remain));
            self.stdout.flush().unwrap();
            self.calculate_fold_points();
            self.move_cursor(-(remain.len() as i32) + 1);
            return;
        }
    }

    fn end(&mut self, text: &str) {
        write!(self.stdout, "{}", text).unwrap();
    }
}

pub fn prompt_normal(_core: &mut ShellCore) -> u16 {
    let host = "🍣";

    print!("{} ", host);
    io::stdout().flush().unwrap();

    (chars_to_width(&host.chars().collect()) + 1 ) as u16
}

pub fn read_line_terminal(left: u16, core: &mut ShellCore) -> Option<String>{
    let mut writer = Writer::new(core.history.len(), left);

    for c in stdin().keys() {
        match &c.as_ref().unwrap() {
            event::Key::Ctrl('a') => writer.move_cursor_to_head(),
            event::Key::Ctrl('b') => writer.move_cursor(-1),
            event::Key::Ctrl('c') => {
                writer.chars.clear();
                writer.end("^C\r\n");
                return None;
            },
            event::Key::Ctrl('e') => writer.move_cursor_to_tail(),
            event::Key::Ctrl('f') => writer.move_cursor(1),
            event::Key::Char('\n') => {
                writer.end("\r\n");
                break;
            },
            event::Key::Up         => writer.call_history(-1, &core.history),
            event::Key::Down       => writer.call_history(1, &core.history),
            event::Key::Left       => writer.move_cursor(-1),
            event::Key::Right      => writer.move_cursor(1),
            event::Key::Backspace  => writer.remove(),
            event::Key::Char(ch)    => writer.insert(*ch),
            _  => {},
        }

    }

    let ans = chars_to_string(&writer.chars);
    if ans.len() != 0 {
        core.history.push(ans.clone());
    };
    Some(ans + "\n")
}
