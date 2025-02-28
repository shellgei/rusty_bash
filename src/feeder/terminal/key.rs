//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::input::InputError;
use std::sync::atomic::Ordering::Relaxed;
use super::Terminal;
use termion::event;
use termion::event::Key;

pub fn action(core: &mut ShellCore, term: &mut Terminal, c: &Key) -> Result<bool, InputError> {
    match c {
        event::Key::Ctrl(ch) => ctrl(core, term, *ch)?,
        event::Key::Down |
        event::Key::Left |
        event::Key::Right |
        event::Key::Up => arrow(term, core, c),
        event::Key::Backspace => term.backspace(),
        event::Key::Delete => term.delete(),
        event::Key::Char(c) => return char_key(term, core, c),
        _  => {},
    }
    Ok(false)
}

fn ctrl(core: &mut ShellCore, term: &mut Terminal, c: char) -> Result<(), InputError>{
    match c {
        'a' => term.goto_origin(),
        'b' => term.shift_cursor(-1),
        'c' => {
            core.sigint.store(true, Relaxed);
            term.goto(term.chars.len());
            term.write("^C\r\n");
            return Err(InputError::Interrupt);
        },
        'd' => {
            if term.chars.len() == term.prompt.chars().count() {
                term.write("\r\n");
                return Err(InputError::Eof);
            }else{
                term.delete();
            }
        },
        'e' => term.goto_end(),
        'f' => term.shift_cursor(1),
        _ => {},
    }
    Ok(())
}

fn arrow(term: &mut Terminal, core: &mut ShellCore, key: &event::Key) {
    if term.tab_num > 1 {
        match key {
            event::Key::Down  => term.tab_row += 1,
            event::Key::Up    => term.tab_row -= 1,
            event::Key::Right => term.tab_col += 1,
            event::Key::Left  => term.tab_col -= 1,
            _ => {},
        }
        let _ = term.completion(core);
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

fn char_key(term: &mut Terminal, core: &mut ShellCore, c: &char)
                                       -> Result<bool, InputError> {
    match c {
        '\n' => {
            if term.completion_candidate.len() > 0 {
                term.set_double_tab_completion(core);
            }else{
                term.goto(term.chars.len());
                term.write("\r\n");
                term.chars.push('\n');
                return Ok(true);
            }
        },
        '\t' => {
            if term.tab_num == 0 || term.prev_key == event::Key::Char('\t') {
                term.tab_num += 1;
            }
            if term.tab_num == 2 {
                term.tab_row = -1;
                term.tab_col = 0;
            }else if term.tab_num > 2 {
                term.tab_row += 1;
            }
            let _ = term.completion(core);
        },
        c => term.insert(*c),
    }
    Ok(false)
}
