//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod term;
mod scanner;

use std::io;
use crate::ShellCore;
use std::process;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Clone, Debug)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: "".to_string(),
            backup: vec![],
        }
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let cut = self.remaining[0..cutpos].to_string();
        self.remaining = self.remaining[cutpos..].to_string();

        cut
    }

    pub fn set_backup(&mut self) {
        self.backup.push(self.remaining.clone());
    }

    pub fn pop_backup(&mut self) {
        self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
    }

    pub fn rewind(&mut self) {
        self.remaining = self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
    }   

    /*
    pub fn feed_line(&mut self, core: &mut ShellCore) -> bool {
        let line;
        let len_prompt = term::prompt_normal(core);
        if let Some(ln) = term::read_line_terminal(len_prompt, core) {
            line = ln
        }else{
            return false;
        };

        self.add_line(line);

        if self.len_as_chars() < 2 {
            return true;
        }

        true
    }
    */
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

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> bool {
        if core.sigint.load(Relaxed) { //core.input_interrupt {
            return false;
        }

        let ret = if core.has_flag('i') {
            let len_prompt = term::prompt_additional();
            if let Some(s) = term::read_line_terminal(len_prompt, core){
                Some(s)
            }else {
                return false;
            }
        }else{
            Self::read_line_stdin()
        };

        if let Some(line) = ret {
            self.add_line(line.clone());

            for b in self.backup.iter_mut() {
                if b.ends_with("\\\n") {
                    b.pop();
                    b.pop();
                }
                *b += &line.clone();
            }
        }else{
            eprintln!("sush: syntax error: unexpected end of file");
            process::exit(2);
        }
        true
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

        /*
        while self.remaining.ends_with("\\\n") {
            self.remaining.pop();
            self.remaining.pop();
            if !self.feed_additional_line(core){
                self.remaining = "".to_string();
                return true;
            }
        }*/
        true
    }

    fn add_line(&mut self, line: String) {
        //self.to_lineno += 1;

        if self.remaining.len() == 0 {
            /*
            self.from_lineno = self.to_lineno;
            self.pos_in_line = 0;
            */
            self.remaining = line;
        }else{
            self.remaining += &line;
        };
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }
}
