//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;

mod parser;
mod elements;

use parser::ReadingText;
use elements::CommandWithArgs;

fn prompt(text: &ReadingText) {
    print!("{} $ ", text.to_lineno+1);
    io::stdout()
        .flush()
        .unwrap();
}

fn read_line(text: &mut ReadingText) {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

    text.to_lineno += 1;

    if text.remaining.len() == 0 {
        text.from_lineno = text.to_lineno;
        text.pos_in_line = 0;
        text.remaining = line;
    }else{
        text.remaining += &line;
    };
}

fn main() {
    let mut input = ReadingText{
        remaining: "".to_string(),
        from_lineno: 0,
        to_lineno: 0,
        pos_in_line: 0,
    };

    loop {
        prompt(&input);
        read_line(&mut input);
        let elem = parser::top_level_element(&mut input);
        if let Ok(e) = elem.downcast::<CommandWithArgs>() {
            e.exec();
        };
    }
}

#[test]
fn parse() -> () {
    let elem = parser::top_level_element("echo hoge".to_string());
    if let Ok(e) = elem.downcast::<CommandWithArgs>(){
        assert_eq!(e.args[0].text, "echo");
        assert_eq!(e.args[1].text, "hoge");
    }else{
        panic!("not parsed as a command");
    }
}

#[test]
fn command_test() -> (){
}
