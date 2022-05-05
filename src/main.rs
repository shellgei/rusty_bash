//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;
use std::process::exit;
use std::path::Path;
use std::os::linux::fs::MetadataExt;

mod parser;
mod elements;
mod system;

use parser::ReadingText;
use system::Config;
use std::process;
use crate::elements::BashElem;


fn prompt(text: &ReadingText) {
    print!("{} $ ", text.to_lineno+1);
    io::stdout()
        .flush()
        .unwrap();
}

fn read_line(text: &mut ReadingText) {
    let mut line = String::new();

    let len = io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

    if len == 0 {
        exit(0);
    }

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
    let mut input = ReadingText{
        remaining: "".to_string(),
        from_lineno: 0,
        to_lineno: 0,
        pos_in_line: 0,
    };

    let mut config = Config::new();

    let pid = process::id();
    config.vars.insert("PID", pid.to_string());
    config.flags.i = is_interactive(pid);

    loop {
        if config.flags.i {
            prompt(&input);
        };
        read_line(&mut input);
        parser::top_level_element(&mut input, &mut config).exec();
    }
}

/*
#[test]
fn parse() -> () {
    let mut input = ReadingText{
        remaining: "echo hoge\n".to_string(),
        from_lineno: 1,
        to_lineno: 1,
        pos_in_line: 0,
    };

    let elem = parser::top_level_element(&mut input);
    if let Ok(e) = elem.downcast::<CommandWithArgs>(){
        assert_eq!(e.args[0].text, "echo");
        assert_eq!(e.args[1].text, "hoge");
    }else{
        panic!("not parsed as a command");
    }
}
*/
