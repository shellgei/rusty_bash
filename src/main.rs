//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;
use std::io::Write;
use std::ffi::CString;

use nix::unistd::{execvp, fork, ForkResult, Pid}; 
use nix::sys::wait::*;

mod parser {
    use std::ffi::CString;

    // job or function comment or blank (finally) 
    pub fn top_level_element(line: String) -> (bool, Box<[CString]>) {
        //only a command is recognized currently
        let words: Vec<CString> = line
            .trim()
            .split(" ")
            .map(|x| CString::new(x).unwrap())
            .collect::<Vec<_>>();
    
        let array = words.into_boxed_slice();
        (true, array)
    }
}

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

fn exec(array: Box<[CString]>) {
    execvp(&array[0], &*array).expect("Cannot exec");
}

fn run_ext_command(array: Box<[CString]>) {

    unsafe {
      match fork() {
          Ok(ForkResult::Child) => exec(array),
          Ok(ForkResult::Parent { child } ) => wait_ext_command(child),
          Err(err) => panic!("Failed to fork. {}", err),
      }
    }
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
    };
}

fn main() {
    loop {
        prompt();
        let line = read_line();
        let ans = parser::top_level_element(line);
        run_ext_command(ans.1);
    }
}


#[test]
fn parse() -> () {
    let ans = parser::top_level_element("echo hoge".to_string());
    assert_eq!(ans.1[0], CString::new("echo").unwrap());
    assert_eq!(ans.1[1], CString::new("hoge").unwrap());
}
