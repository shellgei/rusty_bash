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
    prompt_row: usize,
    chars: Vec<char>,
    head: usize,
    prev_size: (usize, usize),
    prev_exlines: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut sout = io::stdout().into_raw_mode().unwrap();
        let row = sout.cursor_pos().unwrap().1;

        Terminal {
            prompt: prompt.to_string(),
            stdout: sout,
            prompt_row: row as usize,
            chars: prompt.chars().collect(),
            head: prompt.chars().count(),
            prev_size: Terminal::size(),
            prev_exlines: 0,
        }
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

    fn cursor_pos(&self, head: usize, y_origin: usize) -> (usize, usize) {
        let col = Terminal::size().0;
        let (mut x, mut y) = (0, y_origin);

        for c in &self.chars[..head] {
            let w = Self::char_width(c);
            if x + w > col {
                y += 1;
                x = w;
            }else{
                x += w;
            }
        }

        (x + 1, y)
    }

    fn goto(&mut self, head: usize) {
        let pos = self.cursor_pos(head, self.prompt_row);
        let size = Terminal::size();

        let x: u16 = std::cmp::min(size.0, pos.0).try_into().unwrap();
        let y: u16 = std::cmp::min(size.1, pos.1).try_into().unwrap();
        self.write(&termion::cursor::Goto(x, y).to_string());
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
        let extra_lines = self.cursor_pos(self.chars.len(), 0).1;
        let row = Terminal::size().1;

        if self.prompt_row + extra_lines > row {
            let ans = row as i32 - extra_lines as i32;
            self.prompt_row = std::cmp::max(ans, 1) as usize;
        }
    }

    pub fn check_size_change(&mut self) {
        if self.prev_size == Terminal::size() {
            return;
        }
        self.prev_size = Terminal::size();

        let ex_lines = self.cursor_pos(self.chars.len(), 0).1;
        let diff = ex_lines as i32 - self.prev_exlines as i32;
        if diff >= 0 {
            self.prompt_row = std::cmp::max(self.prompt_row as i32 - diff, 1) as usize;
        }

        self.goto(0);
        self.write(&termion::clear::AfterCursor.to_string());
        self.write(&self.chars.iter().collect::<String>());
        self.goto(self.head);
        self.flush();
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
        term.check_size_change();
        term.prev_exlines = term.cursor_pos(term.chars.len(), 0).1;
    }
    Ok(term.get_string(term.prompt.chars().count()))
}
