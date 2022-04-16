//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;
use std::ffi::CString;

use nix::unistd::{execvp, fork, ForkResult, Pid}; 
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
    let words: Vec<CString> = line
        .trim()
        .split(" ")
        .map(|x| CString::new(x).unwrap())
        .collect::<Vec<_>>();

    let array = words.into_boxed_slice();

    //execvp(&args[0], &[&args[0], &args[1]]).unwrap();
    //execvp(&words[0], &[words.iter().map(|x| x.as_ref()).collect()]).expect("Cannot exec");
    //execvp(&words[0], &*array).expect("Cannot exec");
    execvp(&array[0], &*array).expect("Cannot exec");

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

    unsafe {
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
}

fn main() {
    loop {
        main_loop();
    }
//    process::exit(0);
}
