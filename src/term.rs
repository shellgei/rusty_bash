//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::{Write, stdout, stdin};
use termion::{event};
use termion::cursor::DetectCursorPos;
use termion::raw::IntoRawMode;
use termion::input::TermRead;

pub fn prompt(text: &String) -> usize {
    let prompt = format!("{} $ ", text);
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    prompt.len()
}

fn left_cur_pos(pos: usize) -> usize {
    if pos == 0 {
        0
    }else{
        pos - 1
    }
}

pub fn read_line(left: usize) -> String{
    let mut chars: Vec<char> = vec!();
    let mut widths = vec!();
    let mut cur_pos = 0;

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

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
           //     println!("{:?}", chars);

                if x-widths[cur_pos] > left as u16 {
                    write!(stdout, "{}", termion::cursor::Goto(x-widths[cur_pos], y)).unwrap();
                };
                stdout.flush().unwrap();
            },
            event::Key::Backspace => {
                cur_pos = left_cur_pos(cur_pos);
                chars.remove(cur_pos);

                write!(stdout, "{}{}",
                       termion::cursor::Goto(x-widths[cur_pos], y), termion::clear::UntilNewline )
                    .unwrap();
                stdout.flush().unwrap();
            },
            event::Key::Char(c) => {
                    write!(stdout, "{}", c).unwrap();
                    if c == '\n' {
                        chars.push(c);
                        break;
                    };

                    chars.insert(cur_pos, c);
                    cur_pos += 1;

                    /*
                    if chars.len() != cur_pos {
                        let right = chars[cur_pos+1..].iter().collect::<String>();
                        write!(stdout, "{}{}{}{}", 
                               termion::cursor::Goto(x, y),
                               termion::clear::UntilNewline,
                               right,
                               termion::cursor::Goto(x+chars[cur_pos], y)
                               ).unwrap();
                    };
                    */
                    stdout.flush().unwrap();
                    let (new_x, _) = stdout.cursor_pos().unwrap();
                    widths.push(new_x - x);
            },
            _ => {},
        }
    }
    write!(stdout, "\r").unwrap();
    stdout.flush().unwrap();
    chars.iter().collect::<String>()
}

