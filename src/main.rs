//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod parser;
mod elements;
mod core;
mod term;

use std::{io,env,process};
use std::process::exit;
use std::path::Path;
use std::os::linux::fs::MetadataExt;

use crate::core::ShellCore;
use crate::elements::BashElem;
use crate::parser::ReadingText;

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

fn main() {
    let mut core = ShellCore::new();
    let args: Vec<String> = env::args().collect();

    for arg in &args {
        if arg == "-d" {
            core.flags.d = true;
        };
    };

    let mut input = ReadingText{
        remaining: "".to_string(),
        from_lineno: 0,
        to_lineno: 0,
        pos_in_line: 0,
    };


    let pid = process::id();
    core.vars.insert("PID", pid.to_string());
    core.flags.i = is_interactive(pid);

    loop {
        let line = if core.flags.i {
            let len_prompt = term::prompt(&format!("{}", input.to_lineno+1));
            term::read_line(len_prompt, &core.history)
        }else{
            read_line()
        };
        core.history.push(line.trim_end().to_string());
        add_line(&mut input, line);
        parser::top_level_element(&mut input, &mut core).exec(&mut core);
    }
}
