//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;
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
            *b += &line;
        }
    }

    pub fn rewind(&mut self) {
        self.remaining = self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
    }   

    fn read_line_stdin() -> Result<String, InputError> {
        let mut line = String::new();

        let len = io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        if len == 0 {
            Err(InputError::Eof)
        }else{
            Ok(line)
        }
    }

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let line = match core.has_flag('i') {
            true  => terminal::read_line(core, "PS2"),
            false => Self::read_line_stdin(),
        };

        match line { 
            Ok(ln) => {
                self.add_line(ln.clone());
                self.add_backup(&ln);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> bool {
        match self.feed_additional_line_core(core) {
            Ok(()) => true,
            Err(InputError::Eof) => {
                eprintln!("sush: syntax error: unexpected end of file");
                core.set_param("?", "2");
                core.exit();
            },
            Err(InputError::Interrupt) => {
                core.set_param("?", "130");
                false
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let pwd = core.get_param_ref("PWD").to_string();
        core.set_param("PS1", &(pwd + "🍣 "));

        let line = match core.has_flag('i') {
            true  => terminal::read_line(core, "PS1"),
            false => Self::read_line_stdin(),
        };

        match line {
            Ok(ln) => {
                self.add_line(ln);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    fn add_line(&mut self, line: String) {
        match self.remaining.len() {
            0 => self.remaining = line,
            _ => self.remaining += &line,
        };
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }
}
