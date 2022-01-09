//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;

fn prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read line");
    return line;
}

fn main() {

    prompt();
    let line = read_line();

    println!("{}", line);
}
