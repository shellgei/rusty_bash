//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;
mod scanner;

use std::{io, process};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use crate::ShellCore;
use crate::error::input::InputError;
use crate::error::parse::ParseError;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Default)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
    pub nest: Vec<(String, Vec<String>)>,
    pub lineno: usize,
    script_lines: Option<Lines<BufReader<File>>>,
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

    pub fn set_file(&mut self, s: &str) {
        let file = File::open(s).unwrap();
        self.script_lines = Some(BufReader::new(file).lines());
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let tail = self.remaining.split_off(cutpos);
        let ans = self.remaining.to_string();
        self.remaining = tail;
        self.lineno += ans.chars().filter(|c| *c == '\n').count();
        ans
    }

    pub fn refer(&mut self, cutpos: usize) -> &str {
        &self.remaining[0..cutpos]
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

    fn read_script(&mut self, core: &mut ShellCore) -> Result<String, InputError> {
        if let Some(lines) = self.script_lines.as_mut() {
            match lines.next() {
                Some(Ok(line)) => return Ok(line + "\n"),
                _ => return Err(InputError::Eof),
            }
        }

        let mut line = String::new();
        let len = match io::stdin().read_line(&mut line) {
            Ok(len)  => len,
            Err(why) => {
                eprintln!("sush: {}: {}", &core.script_name, why);
                process::exit(1)
            },
        };

        match len  {
            0 => Err(InputError::Eof),
            _ => Ok(line),
        }
    }

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let line = match ! core.read_stdin && self.script_lines.is_none() {
            true  => terminal::read_line(core, "PS2"),
            false => self.read_script(core),
        };

        line.map(|ln| {
            self.add_line(ln.clone(), core);
            self.add_backup(&ln);
        })
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> Result<(), ParseError> {
        match self.feed_additional_line_core(core) {
            Ok(()) => Ok(()),
            Err(InputError::Eof) => {
                core.db.exit_status = 2;
                return Err(ParseError::Input(InputError::Eof));
            },
            Err(InputError::Interrupt) => {
                core.db.exit_status = 130;
                Err(ParseError::Input(InputError::Interrupt))
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = match ! core.read_stdin && self.script_lines.is_none() {
            true  => terminal::read_line(core, "PS1"),
            false => self.read_script(core),
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
        self.remaining = to.to_string() + &self.remaining;
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }

    pub fn nth(&self, n: usize) -> Option<char> {
        self.remaining.chars().nth(n)
    }
}
