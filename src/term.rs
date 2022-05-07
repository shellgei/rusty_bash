//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin};
use std::convert::TryInto;
use std::io::Stdout;

use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::{IntoRawMode, RawTerminal};
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

fn cur_move(x: u16, y: u16, stdout: &mut RawTerminal<Stdout>){
    write!(stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
    stdout.flush().unwrap();
}

fn rewrite_line(left: u16, y: u16, text: String, stdout: &mut RawTerminal<Stdout>){
    write!(stdout, "{}{}{}",
           termion::cursor::Goto(left+1, y),
           termion::clear::UntilNewline,
           text).unwrap();
    stdout.flush().unwrap();
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
                if x-widths[cur_pos] > left {
                    cur_move(x-widths[cur_pos], y, &mut stdout);
                };
            },
            event::Key::Right => {
                if chars.len() > cur_pos+1 {
                    cur_pos += 1;
                    cur_move(x+widths[cur_pos], y, &mut stdout);
                }else{
                    let line_len = widths.iter().fold(0, |line_len, w| line_len + w);
                    cur_move(left+line_len+1, y, &mut stdout);
                    cur_pos = chars.len();
                };
            },
            event::Key::Backspace => {
                cur_pos = left_cur_pos(cur_pos);
                chars.remove(cur_pos);
                rewrite_line(left, y, chars.iter().collect::<String>(), &mut stdout);
                cur_move(x-widths[cur_pos], y, &mut stdout);
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
                    rewrite_line(left, y, chars[0..cur_pos].iter().collect::<String>(), &mut stdout);
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

