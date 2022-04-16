//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;
use std::ffi::CString;

use nix::unistd::{execv, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

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

fn run_ext_command(line: String) {
    let args: Vec<&str> = line
        .trim()
        .split(" ")
        .collect();

    let dir = CString::new("/bin/echo").unwrap();
    let com = CString::new(args[1]).unwrap();
    println!("{:?}", com);

    execv(&dir, &[&com.clone()]).expect("Failed to execute");
}

fn wait_ext_command(child: Pid) {
    match waitpid(child, None)
        .expect("Faild to wait child process.") {
        WaitStatus::Exited(pid, status) => {
            if status != 0 {
                println!("Pid: {:?}, Exit with {:?}", pid, status);
            };
        }
        WaitStatus::Signaled(pid, signal, _) => {
            println!("Pid: {:?}, Signal: {:?}", pid, signal)
        }
        _ => {
            println!("Unknown error")
        }
    }
}

fn main_loop() {
    prompt();
    let line = read_line();

    match fork() {
        Ok(ForkResult::Child) => {
            run_ext_command(line)
        }
        Ok(ForkResult::Parent { child } ) => {
            wait_ext_command(child)
        }
        Err(err) => panic!("Failed to fork. {}", err)
    }
}

fn main() {
    loop {
        main_loop();
    }
//    process::exit(0);
}
