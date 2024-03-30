//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, Stdout};
use termion::cursor::DetectCursorPos;
use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use unicode_width::UnicodeWidthStr;

struct Terminal {
    prompt: String,
    stdout: RawTerminal<Stdout>,
    chars: Vec<char>,
    insert_pos: usize,
    original_row: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        let mut term = Terminal {
            prompt: prompt.to_string(),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: prompt.chars().collect(),
            insert_pos: prompt.chars().count(),
            original_row: 0,
        };

        print!("{}", prompt);
        term.flush();
        term.original_row = term.stdout.cursor_pos().unwrap().1 as usize;

        term
    }

    fn char_width(c: &char) -> usize {
         UnicodeWidthStr::width(c.to_string().as_str())
    }
    
    fn size() -> (usize, usize) {
        let (c, r) = termion::terminal_size().unwrap();
        (c as usize, r as usize)
    }

    fn cursor_pos(&self, ins_pos: usize) -> (usize, usize) {
        let (col, _) = Self::size();
        let mut x = 0;
        let mut y = 0;
        for c in &self.chars[..ins_pos] {
            let w = Self::char_width(c);
            if x + w > col {
                x = w;
                y += 1;
            }else{
                x += w;
            }
        }
        (x + 1, y + self.original_row)
    }

    fn goto(&mut self, char_pos: usize) {
        let pos = self.cursor_pos(char_pos);
        self.write(
            &termion::cursor::Goto(
                pos.0.try_into().unwrap(),
                pos.1.try_into().unwrap()
            ).to_string()
        );
    }

    
    fn write(&mut self, s: &str) {
        write!(self.stdout, "{}", s).unwrap();
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.insert_pos, c);
        self.insert_pos += 1;

        self.goto(self.prompt.chars().count());
        self.write(&self.get_string());
        self.goto(self.insert_pos);
        self.flush();
    }

    pub fn get_string(&self) -> String {
        let cut = self.prompt.chars().count();
        self.chars[cut..].iter().collect()
    }

    pub fn goto_origin(&mut self) {
        self.insert_pos = self.prompt.chars().count();
        self.goto(self.insert_pos);
        self.flush();
    }

    fn count_lines(&self) -> usize {
        let (col, _) = Self::size();

        let mut len = 0;
        let mut lines = 1;
        for c in &self.chars {
            let x = Self::char_width(c);
            if len + x > col {
                lines += 1;
                len = x;
            } else {
                len += x;
            }
        }
        lines
    }

    pub fn check_scroll(&mut self) {
        let lines = self.count_lines();
        let (_, row) = Self::size();

        if self.original_row + lines - 1 > row {
            self.original_row = row - lines + 1;
        }
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);

    for c in io::stdin().keys() {
        term.check_scroll();

        match c.as_ref().unwrap() {
            event::Key::Ctrl('a') => term.goto_origin(),
            event::Key::Ctrl('c') => {
                write!(term.stdout, "^C\r\n").unwrap();
                return Err(InputError::Interrupt);
            },
            event::Key::Ctrl('d') => {
                write!(term.stdout, "\r\n").unwrap();
                return Err(InputError::Eof);
            },
            event::Key::Char('\n') => {
                write!(term.stdout, "\r\n").unwrap();
                term.chars.push('\n');
                break;
            },
            event::Key::Char(c) => {
                term.insert(*c);
            },
            _  => {},
        }
    }
    Ok(term.get_string())
}
