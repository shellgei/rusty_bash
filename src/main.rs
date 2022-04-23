//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;

mod parser;
mod elements;

use elements::CommandWithArgs;

fn prompt() {
    print!("$ ");
    io::stdout()
        .flush()
        .unwrap();
}

fn read_line() -> String {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    line
}

fn main() {
    loop {
        prompt();
        let line = read_line();
        let elem = parser::top_level_element(line);
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
