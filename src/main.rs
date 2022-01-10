//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;
use std::process::Command;
use std::process;

use nix::unistd::{fork, ForkResult}; 
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

    match fork() {
        Ok(ForkResult::Child) => {
            let args: Vec<&str> = line.trim().split(" ").collect();
            Command::new(args[0]).args(&args[1..]).spawn().expect("Failed to run the command");
        }
        Ok(ForkResult::Parent { child: _, .. }) => {
            process::exit(0);
        }
        Err(err) => panic!("Failed to fork. {}", err),
    }
}
