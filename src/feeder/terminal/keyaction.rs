//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::input::InputError;
use std::sync::atomic::Ordering::Relaxed;
use super::Terminal;
use termion::event;
use termion::event::Key;

pub fn ctrl(core: &mut ShellCore, term: &mut Terminal, c: char) -> Result<(), InputError>{
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

pub fn arrow(term: &mut Terminal, core: &mut ShellCore, key: &event::Key, tab_num: usize) {
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

pub fn char(term: &mut Terminal, core: &mut ShellCore,
            c: &char, tab_num: &mut usize, prev_key: &Key) -> bool {
    match c {
        '\n' => {
            if term.completion_candidate.len() > 0 {
                term.set_double_tab_completion();
            }else{
                term.goto(term.chars.len());
                term.write("\r\n");
                term.chars.push('\n');
                return true;
            }
        },
        '\t' => {
            if *tab_num == 0 || *prev_key == event::Key::Char('\t') {
                *tab_num += 1;
            }
            if *tab_num == 2 {
                term.tab_row = -1;
                term.tab_col = 0;
            }else if *tab_num > 2 {
                term.tab_row += 1;
            }
            term.completion(core, *tab_num);
        },
        c => term.insert(*c),
    }
    false
}
