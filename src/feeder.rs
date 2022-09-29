//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
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
    pub remaining: String,
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

    pub fn consume(&mut self, cutpos: usize) -> String {
        let cut = self.remaining[0..cutpos].to_string(); // TODO: this implementation will cause an error.
        self.pos_in_line += cutpos as u32;
        self.remaining = self.remaining[cutpos..].to_string();

        cut
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> bool {
        //let line = if core.flags.i {
        let line = if true {
            let len_prompt = term::prompt_normal(core);
            if let Some(ln) = term::read_line_terminal(len_prompt, core) {
                ln
            }else{
                return false;
            }
        }else{
            if let Some(s) = read_line_stdin() {
                s
            }else{
                return false;
            }
        };
        self.add_line(line);

        if self.len_as_chars() < 2 {
            return true;
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

    pub fn _text(&self) -> String {
        self.remaining.clone()
    }

    pub fn len_as_chars(&self) -> usize {
        self.remaining.chars().count()
    }

}

