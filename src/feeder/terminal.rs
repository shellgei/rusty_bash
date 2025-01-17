//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod completion;

use crate::{file_check, ShellCore};
use crate::utils::file;
use crate::error::input::InputError;
use std::io;
use std::fs::File;
use std::io::{Write, Stdout};
use std::sync::atomic::Ordering::Relaxed;
use std::path::Path;
use nix::unistd;
use nix::unistd::User;
use termion::cursor::DetectCursorPos;
use termion::event;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use unicode_width::UnicodeWidthChar;

struct Terminal {
    prompt: String,
    stdout: RawTerminal<Stdout>,
    prompt_row: usize,
    chars: Vec<char>,
    head: usize,
    hist_ptr: usize,
    prompt_width_map: Vec<usize>,
    /* for extended completion */
    completion_candidate: String,
    tab_row: i32,
    tab_col: i32,
    escape_at_completion: bool,
}

fn oct_string(s: &str) -> bool {
    if s.chars().nth(0) != Some('\\') {
        return false;
    }

    for i in 1..4 {
        match s.chars().nth(i) {
            Some(c) => {
                if c < '0' || '9' < c {
                    return false;
                }
            },
            _ => return false,
        }
    }

    true
}

fn oct_to_hex_in_str(from: &str) -> String {
    let mut i = 0;
    let mut pos = vec![];

    for ch in from.chars() {
        if oct_string(&from[i..]) {
            pos.push(i);
        }
        i += ch.len_utf8();
    }

    let mut prev = 0;
    let mut ans = String::new();
    for p in pos {
        ans += &from[prev..p];
        if let Ok(n) = u32::from_str_radix(&from[p+1..p+4], 8) {
            ans += &char::from_u32(n).unwrap().to_string();
        }
        prev = p+4;
    }
    ans += &from[prev..];
    ans
}

impl Terminal {
    pub fn new(core: &mut ShellCore, ps: &str) -> Self {
        let raw_prompt = core.db.get_param(ps).unwrap_or(String::new());
        let ansi_on_prompt = oct_to_hex_in_str(&raw_prompt);

        let replaced_prompt = Self::make_prompt_string(&ansi_on_prompt);
        let prompt = replaced_prompt.replace("\\[", "").replace("\\]", "").to_string();
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut sout = io::stdout().into_raw_mode().unwrap();
        let row = sout.cursor_pos().unwrap_or((1,1)).1;

        Terminal {
            prompt: prompt.to_string(),
            stdout: sout,
            prompt_row: row as usize,
            chars: prompt.chars().collect(),
            head: prompt.chars().count(),
            hist_ptr: 0,
            prompt_width_map: Self::make_width_map(&replaced_prompt),
            completion_candidate: String::new(),
            tab_row: -1,
            tab_col: -1,
            escape_at_completion: true,
        }
    }

    fn get_branch(cwd: &String) -> String {
        let mut dirs: Vec<String> = cwd.split("/").map(|s| s.to_string()).collect();
        while dirs.len() > 0 {
            let path = dirs.join("/") + "/.git/HEAD";
            dirs.pop();

            if ! file_check::is_regular_file(&path) {
                continue;
            }

            if let Ok(mut f) = File::open(Path::new(&path)){
                return match f.read_line() {
                    Ok(Some(s)) => s.replace("ref: refs/heads/","") + "🌵",
                    _ => "".to_string(),
                };
            }
        }

        "".to_string()
    }

    fn make_prompt_string(raw: &str) -> String {
        let uid = unistd::getuid();
        let user = match User::from_uid(uid) {
            Ok(Some(u)) => u.name,
            _ => "".to_string(),
        };
        let hostname = match unistd::gethostname() {
            Ok(h) => file::oss_to_name(&h),
            _ => "".to_string(),
        };

        let homedir = match User::from_uid(uid) {
            Ok(Some(u)) => file::buf_to_name(&u.dir),
            _ => "".to_string(),
        };
        let mut cwd = match unistd::getcwd() {
            Ok(p) => file::buf_to_name(&p),
            _ => "".to_string(),
        };
        let branch = Self::get_branch(&cwd);

        if cwd.starts_with(&homedir) {
            cwd = cwd.replacen(&homedir, "~", 1);
        }

        raw.replace("\\u", &user)
           .replace("\\h", &hostname)
           .replace("\\w", &cwd)
           .replace("\\b", &branch)
           .to_string()
    }

    fn make_width_map(prompt: &str) -> Vec<usize> {
        let tmp = prompt.replace("\\[", "\x01").replace("\\]", "\x02").to_string();
        let mut in_escape = false;
        let mut ans = vec![];
        for c in tmp.chars() {
            if c == '\x01' || c == '\x02' {
                in_escape = c == '\x01';
                continue;
            }

            let wid = match in_escape {
                true  => 0,
                false => UnicodeWidthChar::width(c).unwrap_or(0),
            };
            ans.push(wid);
        }
        ans
    }

    fn write(&mut self, s: &str) {
        write!(self.stdout, "{}", s).unwrap();
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    fn char_width(&self, c: &char, pos: usize) -> usize {
        if pos < self.prompt.chars().count() {
            return self.prompt_width_map[pos];
        }

        UnicodeWidthChar::width(*c).unwrap_or(0)
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

        for (i, c) in self.chars[..head].iter().enumerate() {
            if *c == '\n' {
                y += 1;
                x = 0;
                continue;
            }

            let w = self.char_width(c, i);
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
        self.write(&self.get_string(0).replace("\n", "\n\r"));
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
        let prev = self.head;
        Self::shift_in_range(&mut self.head, shift, 
                             self.prompt.chars().count(),
                             self.chars.len());

        if prev == self.head {
            self.cloop();
            return;
        }

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
        self.chars.extend(core.fetch_history(self.hist_ptr, prev, prev_str).replace("↵ \0", "\n").chars());
        self.head = self.chars.len();
        self.rewrite(true);
    }

    pub fn set_double_tab_completion(&mut self) {
        let s = self.completion_candidate.clone() + " ";
        self.replace_input(&s);
    }

    pub fn cloop(&mut self) {
        print!("\x07");
        self.flush();
    }
}

fn is_completion_key(key: event::Key) -> bool {
    match key {
        event::Key::Char('\t') 
            | event::Key::Left | event::Key::Down
            | event::Key::Right | event::Key::Up => true,
        _ => false,
    }
}

fn on_arrow_key(term: &mut Terminal, core: &mut ShellCore, key: &event::Key, tab_num: usize) {
    if tab_num > 1 {
        match key {
            event::Key::Down  => term.tab_row += 1,
            event::Key::Up    => term.tab_row -= 1,
            event::Key::Right => term.tab_col += 1,
            event::Key::Left  => term.tab_col -= 1,
            _ => {},
        }
        term.completion(core, tab_num);
    }else{
        match key {
            event::Key::Down  => term.call_history(-1, core),
            event::Key::Up    => term.call_history(1, core),
            event::Key::Right => term.shift_cursor(1),
            event::Key::Left  => term.shift_cursor(-1),
            _ => {},
        }
    }
}

pub fn read_line(core: &mut ShellCore, prompt: &str) -> Result<String, InputError>{
    let mut term = Terminal::new(core, prompt);
    let mut term_size = Terminal::size();
    core.history.insert(0, String::new());
    let mut prev_key = event::Key::Char('a');
    let mut tab_num = 0;

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
            event::Key::Down |
            event::Key::Left |
            event::Key::Right |
            event::Key::Up => on_arrow_key(&mut term, core, c.as_ref().unwrap(), tab_num),
            event::Key::Backspace => term.backspace(),
            event::Key::Delete => term.delete(),
            event::Key::Char('\n') => {
                if term.completion_candidate.len() > 0 {
                    term.set_double_tab_completion();
                }else{
                    term.goto(term.chars.len());
                    term.write("\r\n");
                    term.chars.push('\n');
                    break;
                }
            },
            event::Key::Char('\t') => {
                if tab_num == 0 || prev_key == event::Key::Char('\t') {
                    tab_num += 1;
                }
                if tab_num == 2 {
                    term.tab_row = -1;
                    term.tab_col = 0;
                }else if tab_num > 2 {
                    term.tab_row += 1;
                }
                term.completion(core, tab_num);
            },
            event::Key::Char(c) => {
                term.insert(*c);
            },
            _  => {},
        }
        term.check_scroll();
        prev_key = c.as_ref().unwrap().clone();
        if ! is_completion_key(prev_key) {
            tab_num = 0;
            term.completion_candidate = String::new();
        }
    }

    core.history[0] = term.get_string(term.prompt.chars().count());
    core.history[0].pop();

    Ok(term.get_string(term.prompt.chars().count()))
}
