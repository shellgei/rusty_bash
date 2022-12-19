//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod scanner;
mod term;

use std::io;
use std::str::Chars;
use crate::ShellCore;
//use crate::term;


#[derive(Clone)]
pub struct Feeder {
    remaining: String,
    from_lineno: u32,
    to_lineno: u32,
    pos_in_line: u32,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: "".to_string(),
            from_lineno: 0,
            to_lineno: 0,
            pos_in_line: 0,
        }
    }

    pub fn new_from(text: String) -> Feeder {
        let mut ans = Feeder::new();
        ans.remaining = text;
        ans
    }

    fn read_line_stdin() -> Option<String> {
        let mut line = String::new();
    
        let len = io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
    
        if len == 0 {
            return None;
        }
        Some(line)
    }

    pub fn lineno(&self) -> (u32, u32) {
        (self.from_lineno, self.to_lineno)
    }

    pub fn pos(&self) -> u32 {
        self.pos_in_line
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }

    pub fn chars_after(&self, s: usize) -> Chars {
        self.remaining[s..].chars()
    }

    pub fn nth(&self, p: usize) -> char {
        if let Some(c) = self.remaining.chars().nth(p){
            c
        }else{
            panic!("Parser error: no {}th character in {}", p, self.remaining)
        }
    }

    pub fn rewind(&mut self, backup: Feeder) {
        self.remaining = backup.remaining.clone();
        self.from_lineno = backup.from_lineno;
        self.to_lineno = backup.to_lineno;
        self.pos_in_line = backup.pos_in_line;
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let cut = self.remaining[0..cutpos].to_string();
        self.pos_in_line += cutpos as u32;
        self.remaining = self.remaining[cutpos..].to_string();

        cut
    }

    pub fn consume_blank(&mut self) -> String {
        let d = self.scanner_blank();
        self.consume(d)
    }

    pub fn consume_comment(&mut self) -> String {
        let mut ans = String::new();

        if self.len() != 0 && self.nth(0) == '#' {
            let d = self.scanner_until(0, "\n");
            ans += &self.consume(d);

            if self.len() != 0 && self.nth(0) == '\n' {
                ans += &self.consume(1);
            }
        }
        ans
    }

    pub fn consume_blank_return(&mut self) -> String {
        let mut ans = "".to_string();
        loop {
            let d = self.scanner_blank();
            if d != 0 {
                ans += &self.consume(d);
            }else if self.remaining.starts_with("\n") {
                ans += &self.consume(1);
            }else{
                break;
            }
        }

        ans
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        self.remaining = self.remaining.replacen(from, to, 1);
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> bool {
        //let ret = if core.flags.i {
        let ret = if core.has_flag('i') {
            let len_prompt = term::prompt_additional();
            if let Some(s) = term::read_line_terminal(len_prompt, core){
                Some(s)
            }else {
                return false;
            }
        }else{
            if let Some(s) = Self::read_line_stdin() {
                Some(s)
            }else{
                return false;
            }
        };

        if let Some(line) = ret {
            self.add_line(line);
            true
        }else{
            false
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> bool {
        //let line = if core.flags.i {
        let line = if core.has_flag('i') {
            let len_prompt = term::prompt_normal(core);
            if let Some(ln) = term::read_line_terminal(len_prompt, core) {
                ln
            }else{
                return false;
            }
        }else{
            if let Some(s) = Self::read_line_stdin() {
                s
            }else{
                return false;
            }
        };
        self.add_line(line);

        if self.len_as_chars() < 2 {
            return true;
        }

        while self.from_to_as_chars(self.len_as_chars()-2, self.len_as_chars()) == "\\\n" {
            self.remaining = self.from_to_as_chars(0, self.len_as_chars()-2);
            if !self.feed_additional_line(core){
                self.remaining = "".to_string();
                return true;
            }
        }
        true
    }

    fn add_line(&mut self, line: String) {
        self.to_lineno += 1;

        if self.remaining.len() == 0 {
            self.from_lineno = self.to_lineno;
            self.pos_in_line = 0;
            self.remaining = line;
        }else{
            self.remaining += &line;
        };
    }

    pub fn request_next_line(&mut self, conf: &mut ShellCore) -> String {
        let t = self.consume_blank_return();
    
        if self.len() == 0 {
            let _ = self.feed_additional_line(conf);
        }

        t
    }

    pub fn rewind_feed_backup(&mut self, backup: &Feeder, conf: &mut ShellCore) -> (Feeder, bool) {
        self.rewind(backup.clone());
        let res = self.feed_additional_line(conf);
        (self.clone(), res)
    }

    /*pub fn nth_is(&self, pos: usize, chars: &str) -> bool{
        let ch = self.nth(pos);
        chars.to_string().find(ch) != None
    }*/

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn _text(&self) -> String {
        self.remaining.clone()
    }

    pub fn from_to(&self, from: usize, to: usize) -> String {
        self.remaining[from..to].to_string()
    }

    pub fn len_as_chars(&self) -> usize {
        self.remaining.chars().count()
    }

    pub fn from_to_as_chars(&self, from: usize, to: usize) -> String {
        self.remaining
            .chars()
            .enumerate()
            .filter(|e| e.0 >= from && e.0 < to)
            .map(|e| e.1)
            .collect()
    }

    /* scanner only used in this file */
    fn scanner_blank(&mut self) -> usize {
        let mut pos = 0;
        for ch in self.chars_after(0) {
            if let Some(_) = " \t".find(ch) {
                pos += ch.len_utf8();
            }else{
                break;
            };
        }
        pos
    }
}

