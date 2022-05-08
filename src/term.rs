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
    pub chars: Vec<char>,
    pub widths: Vec<u8>,
    pub ch_ptr: usize,
    pub hist_ptr: usize,
}

impl Writer {
    pub fn new(hist_size: usize) -> Writer{
        Writer {
            stdout: stdout().into_raw_mode().unwrap(),
            chars: vec!(),
            widths: vec!(),
            ch_ptr: 0,
            hist_ptr: hist_size,
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

    pub fn write_history(&mut self, left: u16, y: u16, inc: i32, history: &Vec<History>){
        if history.len() == 0 {
            return;
        }

        if self.hist_ptr as i32 + inc < 0 {
            self.hist_ptr = 0;
        }else{
            self.hist_ptr = (self.hist_ptr as i32 + inc) as usize;
        }

        if self.hist_ptr >= history.len() {
            self.hist_ptr = history.len();
            return;
        }

        /* THERE MAY BE A BUG */
        let h = &history[self.hist_ptr as usize];
        self.rewrite_line(left, y, h.commandline.to_string());
        self.chars = h.commandline.chars().collect();
        self.widths = h.charwidths.clone();
        self.ch_ptr = self.widths.len();
//        println!("{:?}\n{:?}\n{:?}", self.chars, self.widths, self.ch_ptr);
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
    let mut writer = Writer::new(history.len());

    for c in stdin().keys() {
        let (x, y) = writer.cursor_pos();
        match c.unwrap() {
            event::Key::Ctrl('c') => {
                writer.chars.clear();
                write!(writer.stdout, "^C\n").unwrap();
                break;
            }, event::Key::Up => {
                writer.write_history(left, y, -1, &history);
            },
            event::Key::Down => {
                writer.write_history(left, y, 1, &history);
            },
            event::Key::Left => {
                writer.ch_ptr = left_ch_ptr(writer.ch_ptr);
                if x-writer.widths[writer.ch_ptr] as u16 > left {
                    writer.cur_move(x-writer.widths[writer.ch_ptr] as u16, y);
                };
            },
            event::Key::Right => {
                if writer.chars.len() > writer.ch_ptr+1 {
                    writer.ch_ptr += 1;
                    writer.cur_move(x+writer.widths[writer.ch_ptr] as u16, y);
                }else{
                    let line_len: u16 = writer.widths.iter().fold(0, |line_len, w| line_len + (*w as u16));
                    writer.cur_move(left+line_len+1, y);
                    writer.ch_ptr = writer.chars.len();
                };
            },
            event::Key::Backspace => {
                if writer.chars.len() == 0 {
                    continue;
                };

                writer.ch_ptr = left_ch_ptr(writer.ch_ptr);
                writer.chars.remove(writer.ch_ptr);
                writer.rewrite_line(left, y, writer.chars.iter().collect());

                if x - writer.widths[writer.ch_ptr] as u16 >= left {
                    writer.cur_move(x-writer.widths[writer.ch_ptr] as u16, y);
                }
            },
            event::Key::Char(c) => {
                    if c == '\n' {
                        write!(writer.stdout, "{}", c).unwrap();
                        writer.chars.push(c);
                        break;
                    }
                    writer.chars.insert(writer.ch_ptr, c);
                    writer.ch_ptr += 1;

                    /* output the line before the cursor */
                    writer.rewrite_line(left, y, writer.chars[0..writer.ch_ptr].iter().collect());
                    let (new_x, new_y) = writer.cursor_pos();
                    writer.widths.insert(writer.ch_ptr-1, (new_x - x) as u8);

                    /* output the line after the cursor */
                    write!(writer.stdout, "{}{}",
                           writer.chars[writer.ch_ptr..].iter().collect::<String>(), 
                           termion::cursor::Goto(new_x, new_y),
                    ).unwrap();
                    writer.stdout.flush().unwrap();
            },
            _ => {},
        }
    }
    write!(writer.stdout, "\r").unwrap();
    writer.stdout.flush().unwrap();
    let ans = writer.chars.iter().collect::<String>();

    history.push(History{
        commandline: ans.trim_end().to_string(),
        charwidths: writer.widths});
    ans
}

