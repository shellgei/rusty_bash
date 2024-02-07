//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod term;
mod scanner;

use std::io;
use crate::ShellCore;
use std::sync::atomic::Ordering::Relaxed;

pub enum InputError {
    Interrupt,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
    pub nest: Vec<(String, Vec<String>)>,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: "".to_string(),
            backup: vec![],
            nest: vec![("".to_string(), vec![])],
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

    pub fn add_backup(&mut self, line: &str) {
        for b in self.backup.iter_mut() {
            if b.ends_with("\\\n") {
                b.pop();
                b.pop();
            }
            *b += &line.clone();
        }
    }

    pub fn rewind(&mut self) {
        self.remaining = self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
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

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let ret = if core.has_flag('i') {
            let len_prompt = term::prompt_additional();
            match term::read_line_terminal(len_prompt, core){
                Some(s) => Some(s),
                _       => return Err(InputError::Interrupt),
            }
        }else{
            Self::read_line_stdin()
        };

        if let Some(line) = ret {
            self.add_line(line.clone());
            self.add_backup(&line);
        }else{
            return Err(InputError::Eof);
        }
        Ok(())
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> bool {
        match self.feed_additional_line_core(core) {
            Ok(()) => true,
            Err(InputError::Eof) => {
                eprintln!("sush: syntax error: unexpected end of file");
                core.vars.insert("?".to_string(), 2.to_string());
                core.exit();
            },
            Err(InputError::Interrupt) => {
                core.vars.insert("?".to_string(), 130.to_string());
                false
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> bool {
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
        true
    }

    fn add_line(&mut self, line: String) {
        if self.remaining.len() == 0 {
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
