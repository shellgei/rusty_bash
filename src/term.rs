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

use crate::core::History;

struct Writer {
    pub stdout: RawTerminal<Stdout>, 
}

impl Writer {
    pub fn new() -> Writer{
        Writer {
            stdout: stdout().into_raw_mode().unwrap()
        }
    }

    pub fn cur_move(&mut self, x: u16, y: u16){
        write!(self.stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn rewrite_line(&mut self, left: u16, y: u16, text: String){
        write!(self.stdout, "{}{}{}",
               termion::cursor::Goto(left+1, y),
               termion::clear::UntilNewline,
               text).unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn cursor_pos(&mut self) -> (u16, u16) {
        self.stdout.cursor_pos().unwrap()
    }
}

pub fn prompt(text: &String) -> u16 {
    let prompt = format!("{} $ ", text);
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    prompt.len().try_into().unwrap()
}

fn left_ch_ptr(pos: usize) -> usize {
    if pos == 0 {
        0
    }else{
        pos - 1
    }
}

pub fn read_line(left: u16, history: &mut Vec<History>) -> String{
    let mut chars: Vec<char> = vec!();
    let mut widths: Vec<u8> = vec!();
    let mut ch_ptr = 0;
    let mut hist_ptr = history.len() as i32;

    let mut writer = Writer::new();
    /*
    let mut writer = Writer {
        stdout: stdout().into_raw_mode().unwrap()
    };
    */

    for c in stdin().keys() {
        let (x, y) = writer.cursor_pos();
        match c.unwrap() {
            event::Key::Ctrl('c') => {
                chars.clear();
                write!(writer.stdout, "^C\n").unwrap();
                break;
            },
            event::Key::Up => {
                if history.len() == 0 {
                    continue;
                };

                hist_ptr -= 1;
                if hist_ptr < 0 {
                    hist_ptr = 0;
                };

                let h = &history[hist_ptr as usize];
                writer.rewrite_line(left, y, h.commandline.to_string());
                chars = h.commandline.chars().collect();
                widths = h.charwidths.clone();
                ch_ptr = widths.len();

            },
            event::Key::Down => {
                if history.len() == 0 {
                    continue;
                };

                hist_ptr += 1;
                if history.len() as i32 <= hist_ptr {
                    hist_ptr = history.len() as i32;
                    continue;
                }

                let h = &history[hist_ptr as usize];
                writer.rewrite_line(left, y, h.commandline.to_string());
                chars = h.commandline.chars().collect();
                widths = h.charwidths.clone();
                ch_ptr = widths.len();
            },
            event::Key::Left => {
                ch_ptr = left_ch_ptr(ch_ptr);
                if x-widths[ch_ptr] as u16 > left {
                    writer.cur_move(x-widths[ch_ptr] as u16, y);
                };
            },
            event::Key::Right => {
                if chars.len() > ch_ptr+1 {
                    ch_ptr += 1;
                    writer.cur_move(x+widths[ch_ptr] as u16, y);
                }else{
                    let line_len: u16 = widths.iter().fold(0, |line_len, w| line_len + (*w as u16));
                    writer.cur_move(left+line_len+1, y);
                    ch_ptr = chars.len();
                };
            },
            event::Key::Backspace => {
                if chars.len() == 0 {
                    continue;
                };

                ch_ptr = left_ch_ptr(ch_ptr);
                chars.remove(ch_ptr);
                writer.rewrite_line(left, y, chars.iter().collect());

                if x - widths[ch_ptr] as u16 >= left {
                    writer.cur_move(x-widths[ch_ptr] as u16, y);
                }
            },
            event::Key::Char(c) => {
                    if c == '\n' {
                        write!(writer.stdout, "{}", c).unwrap();
                        chars.push(c);
                        break;
                    }
                    chars.insert(ch_ptr, c);
                    ch_ptr += 1;

                    /* output the line before the cursor */
                    writer.rewrite_line(left, y, chars[0..ch_ptr].iter().collect());
                    let (new_x, new_y) = writer.cursor_pos();
                    widths.insert(ch_ptr-1, (new_x - x) as u8);

                    /* output the line after the cursor */
                    write!(writer.stdout, "{}{}",
                           chars[ch_ptr..].iter().collect::<String>(), 
                           termion::cursor::Goto(new_x, new_y),
                    ).unwrap();
                    writer.stdout.flush().unwrap();
            },
            _ => {},
        }
    }
    write!(writer.stdout, "\r").unwrap();
    writer.stdout.flush().unwrap();
    let ans = chars.iter().collect::<String>();

    history.push(History{
        commandline: ans.trim_end().to_string(),
        charwidths: widths});
    ans
}

