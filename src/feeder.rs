//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;
mod scanner;

use std::io;
use crate::ShellCore;
use crate::error::input::InputError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use std::sync::atomic::Ordering::Relaxed;

/*
pub enum InputError {
    Interrupt,
    Eof,
}*/

#[derive(Clone, Debug, Default)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
    pub nest: Vec<(String, Vec<String>)>,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            nest: vec![("".to_string(), vec![])],
            ..Default::default()
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
            *b += line;
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

        let line = if core.has_flag('i') {
            terminal::read_line(core, "PS2")
        }else{
            Self::read_line_stdin()
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

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> Result<(), ParseError> {
        match self.feed_additional_line_core(core) {
            Ok(()) => Ok(()),
            Err(InputError::Eof) => {
                eprintln!("sush: syntax error: unexpected end of file");
                core.db.set_param("?", "2", None).unwrap();
                exit::normal(core);
                //return Err(ParseError::Input(InputError::Eof));
            },
            Err(InputError::Interrupt) => {
                core.db.set_param("?", "130", None).unwrap();
                Err(ParseError::Input(InputError::Interrupt))
            },
            Err(_) => {
                Err(ParseError::Input(InputError::History))
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = if core.has_flag('i') {
            terminal::read_line(core, "PS1")
        }else{ 
            Self::read_line_stdin()
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
