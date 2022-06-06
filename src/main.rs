//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod abst_arg_elem;
mod abst_command_elem;
mod elem_arg;
mod elem_arg_delimiter;
mod elem_end_of_command;
mod elem_script;
mod elem_blankpart;
mod elem_command;
mod elem_redirect;
mod elem_setvars;
mod elem_subarg_braced;
mod elem_subarg_command_expansion;
mod elem_subarg_double_quoted;
mod elem_subarg_non_quoted;
mod elem_subarg_single_quoted;
mod elem_subarg_variable;
mod elem_substitution;
mod elem_varname;
mod core;
mod term;
mod utils;
mod feeder;
mod scanner;
mod debuginfo;
mod term_completion;

use std::{io,env,process};
use std::process::exit;
use std::path::Path;
use std::os::linux::fs::MetadataExt;
use std::fs::File;
use std::io::Read;

use crate::core::ShellCore;
use crate::abst_command_elem::{CommandElem};
use crate::feeder::Feeder;

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

/*
fn add_line(text: &mut Feeder, line: String) {
    text.add_line(line);
}
*/

fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match Path::new(&std_path).metadata() {
        Ok(metadata) => metadata.st_mode() == 8592, 
        Err(err) => panic!("{}", err),
    }
}

fn get_hostname() -> String{
    if let Ok(mut file) = File::open("/etc/hostname") {

        let mut fullname = String::new();
        if let Ok(_) = file.read_to_string(&mut fullname) {
            return fullname.trim_end().to_string();
        }
    }

    "unknown".to_string()
}

fn main() {
    let mut core = ShellCore::new();
    let args: Vec<String> = env::args().collect();

    for arg in &args {
        if arg == "-d" {
            core.flags.d = true;
        };
    };

    let mut input = Feeder::new();


    let pid = process::id();
    core.vars.insert("PID".to_string(), pid.to_string());
    core.vars.insert("HOSTNAME".to_string(), get_hostname());
    core.vars.insert("SHELL".to_string(), "rustybash".to_string());
    core.flags.i = is_interactive(pid);

    loop {
        let line = if core.flags.i {
            let len_prompt = term::prompt(&mut core);
            term::read_line(len_prompt, &mut core)
        }else{
            read_line()
        };
        input.add_line(line);
        while let Some(mut e) = elem_script::top_level_element(&mut input){
            e.exec(&mut core);
        }
    }
}
