//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod abst_arg_elem;
mod abst_command_elem;
mod abst_script_elem;
mod elem_arg;
mod elem_arg_delimiter;
mod elem_compound_paren;
mod elem_compound_brace;
mod elem_end_of_command;
mod elem_function;
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
mod elem_subarg_tilde;
mod elem_subarg_variable;
mod elem_substitution;
mod elem_pipeline;
mod elem_varname;
mod core;
mod term;
mod utils;
mod utils_io;
mod feeder;
mod scanner;
mod debuginfo;
mod term_completion;

use std::{env,process};
use std::process::exit;
use std::path::Path;
use std::os::linux::fs::MetadataExt;
use std::fs::File;
use std::io::Read;

use crate::core::ShellCore;
use crate::feeder::Feeder;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufRead;
use crate::elem_command::Command;
use crate::abst_script_elem::ScriptElem;
use crate::elem_script::Script;


fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match Path::new(&std_path).metadata() {
        Ok(metadata) => metadata.st_mode() == 8592, 
        Err(err) => panic!("{}", err),
    }
}

/* This function will be replaced "source" in future. */
fn read_bashrc(core: &mut ShellCore){
    let home = if let Ok(h) = env::var("HOME"){
        h
    }else{
        panic!("Home is not set");
    };

    if let Ok(file) = OpenOptions::new().read(true).open(home + "/.bashrc"){
        let br = BufReader::new(file);
        for ln in br.lines() {
            match ln {
                Ok(mut line) => {
                    line = line.trim_start().to_string();
                    if line.len() < 7 {
                        continue; 
                    };
                    if &line[0..5] == "alias" {
                        let mut f = Feeder::new_with(line);
                        if let Some(mut c) = Command::parse(&mut f, core) {
                            c.exec(core);
                        }
                    }
                },
                _ => break,
            }
        }
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
    /* Ignore Ctrl+C (Childlen will receive instead.) */
    ctrlc::set_handler(move || { })
    .expect("Unable to set the Ctrl+C handler.");

    let mut core = ShellCore::new();
    let args: Vec<String> = env::args().collect();

    core.flags.d = args.iter().any(|a| a.clone() == "-d");

    let pid = process::id();
    core.vars.insert("PID".to_string(), pid.to_string());
    core.vars.insert("HOSTNAME".to_string(), get_hostname());
    core.vars.insert("SHELL".to_string(), "rustybash".to_string());
    core.flags.i = is_interactive(pid);

    read_bashrc(&mut core);

    let mut feeder = Feeder::new();
    loop {
        if !feeder.feed_line(&mut core) {
            break;
        }
        while let Some(mut e) = Script::parse(&mut feeder, &mut core, false){
            e.exec(&mut core);
        }
    }

    if let Ok(status) = core.get_var(&"?".to_string())
                        .to_string().parse::<i32>(){
        exit(status);
    }else{
        eprintln!("Shell internal error");
        exit(1);
    }

}
