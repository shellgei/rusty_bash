//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin};
use std::convert::TryInto;

use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::IntoRawMode;
use termion::input::TermRead;

pub fn prompt(text: &String) -> u16 {
    let prompt = format!("{} $ ", text);
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    prompt.len().try_into().unwrap()
}

fn left_cur_pos(pos: usize) -> usize {
    if pos == 0 {
        0
    }else{
        pos - 1
    }
}

pub fn read_line(left: u16) -> String{
    let mut chars: Vec<char> = vec!();
    let mut widths = vec!();
    let mut cur_pos = 0;

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    for c in stdin.keys() {
        let (x, y) = stdout.cursor_pos().unwrap();
        match c.unwrap() {
            event::Key::Ctrl('c') => {
                chars.clear();
                write!(stdout, "^C\n").unwrap();
                break;
            },
            event::Key::Left => {
                cur_pos = left_cur_pos(cur_pos);
                if x-widths[cur_pos] > left as u16 {
                    write!(stdout, "{}", termion::cursor::Goto(x-widths[cur_pos], y)).unwrap();
                };
                stdout.flush().unwrap();
            },
            event::Key::Right => {
                if chars.len() > cur_pos+1 {
                    cur_pos += 1;
                    write!(stdout, "{}", termion::cursor::Goto(x+widths[cur_pos], y)).unwrap();
                    stdout.flush().unwrap();
                }else{
                    write!(stdout, "{}{}{}",
                           termion::cursor::Goto(left+1, y),
                           termion::clear::UntilNewline,
                           chars.iter().collect::<String>(), 
                    ).unwrap();
                    stdout.flush().unwrap();
                    cur_pos = chars.len();
                };
            },
            event::Key::Backspace => {
                cur_pos = left_cur_pos(cur_pos);
                chars.remove(cur_pos);
                write!(stdout, "{}{}{}{}",
                       termion::cursor::Goto(left+1, y),
                       termion::clear::UntilNewline,
                       chars.iter().collect::<String>(), 
                       termion::cursor::Goto(x-widths[cur_pos], y))
                    .unwrap();

                stdout.flush().unwrap();
            },
            event::Key::Char(c) => {
                    if c == '\n' {
                        write!(stdout, "{}", c).unwrap();
                        chars.push(c);
                        break;
                    }
                    chars.insert(cur_pos, c);
                    cur_pos += 1;

                    /* output the line before the cursor */
                    write!(stdout, "{}{}{}",
                           termion::cursor::Goto(left+1, y),
                           termion::clear::UntilNewline,
                           chars[0..cur_pos].iter().collect::<String>(), 
                    ).unwrap();
                    stdout.flush().unwrap();

                    let (new_x, new_y) = stdout.cursor_pos().unwrap();
                    widths.insert(cur_pos-1, new_x - x);

                    /* output the line after the cursor */
                    write!(stdout, "{}{}",
                           chars[cur_pos..].iter().collect::<String>(), 
                           termion::cursor::Goto(new_x, new_y),
                    ).unwrap();
                    stdout.flush().unwrap();
            },
            _ => {},
        }
    }
    write!(stdout, "\r").unwrap();
    stdout.flush().unwrap();
    chars.iter().collect::<String>()
}

