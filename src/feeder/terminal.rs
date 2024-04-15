//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::{Write, Stdout};
use std::sync::atomic::Ordering::Relaxed;
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
    hist_ptr: usize,
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let prompt = core.get_param_ref(ps);
        print!("\x1b[1;35m{}\x1b[m", prompt);
        io::stdout().flush().unwrap();

        let mut sout = io::stdout().into_raw_mode().unwrap();
        let row = sout.cursor_pos().unwrap().1;

        Terminal {
            prompt: prompt.to_string(),
            stdout: sout,
            prompt_row: row as usize,
            chars: prompt.chars().collect(),
            head: prompt.chars().count(),
            hist_ptr: 0,
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

    fn shift_in_range(x: &mut usize, shift: i32, min: usize, max: usize) {
        *x = if      shift < 0 && *x < min + (- shift as usize) { min }
             else if shift > 0 && *x + (shift as usize) > max   { max }
             else           { (*x as isize + shift as isize) as usize };
    }

    fn head_to_cursor_pos(&self, head: usize, y_origin: usize) -> (usize, usize) {
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
        let pos = self.head_to_cursor_pos(head, self.prompt_row);
        let size = Terminal::size();

        let x: u16 = std::cmp::min(size.0, pos.0).try_into().unwrap();
        let y: u16 = std::cmp::min(size.1, pos.1).try_into().unwrap();
        self.write(&termion::cursor::Goto(x, y).to_string());
    }

    fn rewrite(&mut self, erase: bool) {
        self.goto(0);
        if erase {
            self.write(&termion::clear::AfterCursor.to_string());
        }
        print!("\x1b[1;35m{}\x1b[m", self.prompt);
        self.write(&self.get_string(self.prompt.chars().count()));
        self.goto(self.head);
        self.flush();
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.head, c);
        self.head += 1;
        self.rewrite(false);
    }

    pub fn backspace(&mut self) {
        if self.head <= self.prompt.chars().count() {
            return;
        }
        self.head -= 1;
        self.chars.remove(self.head);
        self.rewrite(true);
    }

    pub fn delete(&mut self) {
        if self.head >= self.chars.len() {
            return;
        }
        self.chars.remove(self.head);
        self.rewrite(true);
    }

    pub fn get_string(&self, from: usize) -> String {
        self.chars[from..].iter().collect()
    }

    pub fn goto_origin(&mut self) {
        self.head = self.prompt.chars().count();
        self.goto(self.head);
        self.flush();
    }

    pub fn goto_end(&mut self) {
        self.head = self.chars.len();
        self.goto(self.head);
        self.flush();
    }

    pub fn shift_cursor(&mut self, shift: i32) {
        Self::shift_in_range(&mut self.head, shift, 
                             self.prompt.chars().count(),
                             self.chars.len());
        self.goto(self.head);
        self.flush();
    }

    pub fn check_scroll(&mut self) {
        let extra_lines = self.head_to_cursor_pos(self.chars.len(), 0).1;
        let row = Terminal::size().1;

        if self.prompt_row + extra_lines > row {
            let ans = row as isize - extra_lines as isize;
            self.prompt_row = std::cmp::max(ans, 1) as usize;
        }
    }

    pub fn check_size_change(&mut self, prev_size: &mut (usize, usize)) {
        if *prev_size == Terminal::size() {
            return;
        }
        *prev_size = Terminal::size();

        let cur_row = self.stdout.cursor_pos().unwrap().1;
        let diff = self.head_to_cursor_pos(self.head, 0).1;
        let ans = cur_row as isize - diff as isize;
        self.prompt_row = std::cmp::max(ans, 1) as usize;
    }

    pub fn call_history(&mut self, inc: i32, core: &mut ShellCore){
        let prev = self.hist_ptr;
        let prev_str = self.get_string(self.prompt.chars().count());
        Self::shift_in_range(&mut self.hist_ptr, inc, 0, std::isize::MAX as usize);

        self.chars = self.prompt.chars().collect();
        self.chars.extend(core.fetch_history(self.hist_ptr, prev, prev_str).chars());
        self.head = self.chars.len();
        self.rewrite(true);
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);
    let mut term_size = Terminal::size();
    core.history.insert(0, String::new());

    for c in io::stdin().keys() {
        term.check_size_change(&mut term_size);
        match c.as_ref().unwrap() {
            event::Key::Ctrl('a') => term.goto_origin(),
            event::Key::Ctrl('b') => term.shift_cursor(-1),
            event::Key::Ctrl('c') => {
                core.sigint.store(true, Relaxed);
                term.goto(term.chars.len());
                term.write("^C\r\n");
                return Err(InputError::Interrupt);
            },
            event::Key::Ctrl('d') => {
                if term.chars.len() == term.prompt.chars().count() {
                    term.write("\r\n");
                    return Err(InputError::Eof);
                }else{
                    term.delete();
                }
            },
            event::Key::Ctrl('e') => term.goto_end(),
            event::Key::Ctrl('f') => term.shift_cursor(1),
            event::Key::Down => term.call_history(-1, core),
            event::Key::Left => term.shift_cursor(-1),
            event::Key::Right => term.shift_cursor(1),
            event::Key::Up => term.call_history(1, core),
            event::Key::Backspace  => term.backspace(),
            event::Key::Delete  => term.delete(),
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

    core.history[0] = term.get_string(term.prompt.chars().count());
    core.history[0].pop();

    if core.history[0].len() == 0
    || (core.history.len() > 1 && core.history[0] == core.history[1]) {
        core.history.remove(0);
    }

    Ok(term.get_string(term.prompt.chars().count()))
}
