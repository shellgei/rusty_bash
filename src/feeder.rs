//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod scanner;
mod terminal;

use crate::error::input::InputError;
use crate::error::parse::ParseError;
use crate::{utils, ShellCore};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Default)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
    pub nest: Vec<(String, Vec<String>)>,
    pub lineno: usize,
    pub lineno_addition: usize,
    script_lines: Option<Lines<BufReader<File>>>,
    pub main_feeder: bool,
    c_mode_buffer: Vec<String>,
    c_mode: bool,
}

impl Feeder {
    pub fn new(s: &str) -> Feeder {
        Feeder {
            remaining: s.to_string(),
            nest: vec![("".to_string(), vec![])],
            lineno: 1,
            ..Default::default()
        }
    }

    pub fn new_c_mode(s: String) -> Feeder {
        Feeder {
            nest: vec![("".to_string(), vec![])],
            lineno: 1,
            c_mode_buffer: s.split('\n').map(|s| s.to_string()).collect(),
            c_mode: true,
            ..Default::default()
        }
    }

    pub fn set_file(&mut self, s: &str) -> Result<(), InputError> {
        let file = match File::open(s) {
            Ok(f) => f,
            Err(_) => return Err(InputError::NoSuchFile(s.to_string())),
        };
        self.script_lines = Some(BufReader::new(file).lines());
        Ok(())
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let tail = self.remaining.split_off(cutpos);
        let ans = self.remaining.to_string();
        self.remaining = tail;
        let lineno_org = self.lineno;
        self.lineno += ans.chars().filter(|c| *c == '\n').count();

        while self.lineno > lineno_org {
            if self.lineno_addition == 0 {
                break;
            }

            self.lineno_addition -= 1;
            self.lineno -= 1;
        }

        ans
    }

    pub fn refer(&mut self, cutpos: usize) -> &str {
        &self.remaining[0..cutpos]
    }

    pub fn set_backup(&mut self) {
        self.backup.push(self.remaining.clone());
    }

    pub fn pop_backup(&mut self) {
        self.backup
            .pop()
            .expect("SUSHI INTERNAL ERROR (backup error)");
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
        self.remaining = self
            .backup
            .pop()
            .expect("SUSHI INTERNAL ERROR (backup error)");
    }

    fn read_script(&mut self) -> Result<String, InputError> {
        if self.c_mode {
            if self.c_mode_buffer.is_empty() {
                return Err(InputError::Eof);
            }

            return Ok(self.c_mode_buffer.remove(0) + "\n");
        }

        if let Some(lines) = self.script_lines.as_mut() {
            match lines.next() {
                Some(Ok(line)) => return Ok(line + "\n"),
                _ => return Err(InputError::Eof),
            }
        }

        utils::read_line_stdin_unbuffered("")
    }

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if !self.main_feeder {
            return Err(InputError::Eof);
        }

        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let line = match core.db.flags.contains('i') && self.script_lines.is_none() {
            true => terminal::read_line(core, "PS2"),
            false => self.read_script(),
        };

        line.map(|ln| {
            self.add_line(ln.clone(), core);
            self.add_backup(&ln);
        })
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> Result<(), ParseError> {
        match self.feed_additional_line_core(core) {
            Ok(()) => Ok(()),
            Err(InputError::Interrupt) => {
                core.db.exit_status = 130;
                Err(ParseError::Input(InputError::Interrupt))
            }
            Err(e) => {
                core.db.exit_status = 2;
                Err(ParseError::Input(e))
            }
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = match core.db.flags.contains('i') && self.script_lines.is_none() {
            true => terminal::read_line(core, "PS1"),
            false => self.read_script(),
        };

        line.map(|ln| self.add_line(ln, core))
    }

    pub fn add_line(&mut self, line: String, core: &mut ShellCore) {
        if core.db.flags.contains('v') {
            eprint!("{}", &line);
        }

        match self.remaining.len() {
            0 => self.remaining = line,
            _ => self.remaining += &line,
        };
    }

    pub fn replace(&mut self, num: usize, to: &str) {
        self.consume(num);
        self.lineno_addition = to.chars().filter(|c| *c == '\n').count();

        self.remaining = to.to_string() + &self.remaining;
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn starts_withs<T: AsRef<str>>(&self, vs: &[T]) -> bool {
        vs.iter().any(|s| self.remaining.starts_with(s.as_ref()))
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }

    pub fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}
