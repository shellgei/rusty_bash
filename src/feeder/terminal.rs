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
    prev_size: (usize, usize),
    head: usize,
    prompt_row: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        let mut term = Terminal {
            prompt: prompt.to_string(),
            stdout: io::stdout().into_raw_mode().unwrap(),
            chars: prompt.chars().collect(),
            head: prompt.chars().count(),
            prompt_row: 0,
            prev_size: Terminal::size(),
        };

        print!("{}", prompt);
        term.flush();
        term.prompt_row = term.stdout.cursor_pos().unwrap().1 as usize;

        term
    }

    fn write(&mut self, s: &str) {
        write!(self.stdout, "{}", s).unwrap();
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    fn char_width(c: &char) -> usize {
         UnicodeWidthStr::width(c.to_string().as_str())
    }

    fn size() -> (usize, usize) {
        let (c, r) = termion::terminal_size().unwrap();
        (c as usize, r as usize)
    }

    fn cursor_pos(&self, ins_pos: usize, y_origin: usize) -> (usize, usize) {
        let (col, _) = Terminal::size();
        let mut x = 0;
        let mut y = y_origin;

        for c in &self.chars[..ins_pos] {
            let w = Self::char_width(c);
            if x + w > col {
                x = w;
                y += 1;
            }else{
                x += w;
            }
        }

        (x + 1, y)
    }

    fn goto(&mut self, char_pos: usize) {
        let pos = self.cursor_pos(char_pos, self.prompt_row);
        self.write(
            &termion::cursor::Goto(
                pos.0.try_into().unwrap(),
                pos.1.try_into().unwrap()
            ).to_string()
        );
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.head, c);
        self.head += 1;
        self.goto(0);
        self.write(&self.get_string(0));
        self.goto(self.head);
        self.flush();
    }

    pub fn get_string(&self, from: usize) -> String {
        self.chars[from..].iter().collect()
    }

    pub fn goto_origin(&mut self) {
        self.head = self.prompt.chars().count();
        self.goto(self.head);
        self.flush();
    }

    pub fn check_scroll(&mut self) {
        let (_, extra_lines) = self.cursor_pos(self.chars.len(), 0);
        let (_, row) = Terminal::size();

        if self.prompt_row + extra_lines > row {
            if row > extra_lines {
                self.prompt_row = row - extra_lines;
            }else{
                self.prompt_row = 1;
            }
        }

        if self.prev_size != Terminal::size() {
            self.prev_size = Terminal::size();

            self.goto(0);
            write!(self.stdout, "{}", termion::clear::AfterCursor).unwrap();
            self.write(&self.chars.iter().collect::<String>());
            self.goto(self.head);
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
    Ok(term.get_string(term.prompt.chars().count()))
}
