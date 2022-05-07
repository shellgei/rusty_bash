//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use termion::raw::IntoRawMode;
use termion::{event};
use termion::input::TermRead;
use std::io;
use std::io::{Write, stdout, stdin};
use std::process::exit;
use std::path::Path;
use std::os::linux::fs::MetadataExt;
use std::env;

mod parser;
mod elements;
mod core;

use parser::ReadingText;
use crate::core::ShellCore;
use std::process;
use crate::elements::BashElem;


fn prompt(text: &ReadingText) {
    print!("{} $ ", text.to_lineno+1);
    io::stdout()
        .flush()
        .unwrap();
}

fn read_line() -> String {
    let mut line = String::new();

    let len = io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

    if len == 0 {
        exit(0);
    }
    line
}


fn add_line(text: &mut ReadingText, line: String) {
    text.to_lineno += 1;

    if text.remaining.len() == 0 {
        text.from_lineno = text.to_lineno;
        text.pos_in_line = 0;
        text.remaining = line;
    }else{
        text.remaining += &line;
    };
}

fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match Path::new(&std_path).metadata() {
        Ok(metadata) => metadata.st_mode() == 8592, 
        Err(err) => panic!("{}", err),
    }
}

fn read_term_line() -> String{
    let mut line = "".to_string();

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        match c {
            Ok(event::Key::Ctrl('c')) => {
                line = "".to_string();
                write!(stdout, "^C\n").unwrap();
                break;
            },
            Ok(event::Key::Char(c)) => {
                    write!(stdout, "{}", c).unwrap();
                    line += &c.to_string();
                    stdout.flush().unwrap();
                    if c == '\n' {
                        break;
                    };
            },
            _ => {},
        }
    }
    write!(stdout, "\r").unwrap();
    stdout.flush().unwrap();
    line
}

fn main() {
    let mut config = ShellCore::new();
    let args: Vec<String> = env::args().collect();

    for arg in &args {
        if arg == "-d" {
            config.flags.d = true;
        };
    };

    let mut input = ReadingText{
        remaining: "".to_string(),
        from_lineno: 0,
        to_lineno: 0,
        pos_in_line: 0,
    };


    let pid = process::id();
    config.vars.insert("PID", pid.to_string());
    config.flags.i = is_interactive(pid);

    loop {
        let line = if config.flags.i {
            prompt(&input);
            read_term_line()
        }else{
            read_line()
        };
        add_line(&mut input, line);
        parser::top_level_element(&mut input, &mut config).exec(&mut config);
    }
}
