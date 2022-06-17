//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::str::Chars;
use crate::ShellCore;
use crate::term;

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

    pub fn new_with(text: String) -> Feeder {
        let mut ans = Feeder::new();
        ans.remaining = text;
        ans
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

    pub fn replace(&mut self, from: &str, to: &str) {
        self.remaining = self.remaining.replacen(from, to, 1);
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> bool {
        let line = if core.flags.i {
            let len_prompt = term::prompt(core);
            term::read_line_terminal(len_prompt, core)
        }else{
            if let Some(s) = read_line_stdin() {
                s
            }else{
                return false;
            }
        };
        self.add_line(line);
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

    pub fn match_at(&self, pos: usize, chars: &str) -> bool{
        let ch = self.nth(pos);
        chars.to_string().find(ch) != None
    }

    pub fn _text(&self) -> String {
        self.remaining.clone()
    }

    pub fn from_to(&self, from: usize, to: usize) -> String {
        self.remaining[from..to].to_string()
    }
}

