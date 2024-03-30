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
    prev_size: (usize, usize),
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
            prev_size: Terminal::size(),
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
        let (col, _) = Terminal::size();
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
        let (_, y) = self.cursor_pos(self.chars.len());
        y + 1 - self.original_row
    }

    pub fn check_scroll(&mut self) {
        let lines = self.count_lines();
        let (_, row) = Terminal::size();
        if self.original_row + lines - 1 > row {
            let tmp = row as i32 - lines as i32 + 1;
            if tmp < 1 {
                self.original_row = 1;
            }else {
                self.original_row = row - lines + 1;
            }
        }

        if self.prev_size != Terminal::size() {
            self.prev_size = Terminal::size();

            self.goto(0);
            write!(self.stdout, "{}", termion::clear::AfterCursor).unwrap();
            self.write(&self.chars.iter().collect::<String>());
            self.goto(self.insert_pos);
            self.flush();
        }
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);

    for c in io::stdin().keys() {
        match c.as_ref().unwrap() {
            event::Key::Ctrl('a') => term.goto_origin(),
            event::Key::Ctrl('c') => {
                term.goto(term.chars.len());
                term.write("^C\r\n");
                return Err(InputError::Interrupt);
            },
            event::Key::Ctrl('d') => {
                term.write("\r\n");
                return Err(InputError::Eof);
            },
            event::Key::Char('\n') => {
                term.goto(term.chars.len());
                term.write("\r\n");
                term.chars.push('\n');
                break;
            },
            event::Key::Char(c) => {
                term.insert(*c);
            },
            _  => {},
        }
        term.check_scroll();
    }
    Ok(term.get_string())
}
