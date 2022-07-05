//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod abst_elems;
mod bash_glob;
mod elem_arg;
mod elem_compound_paren;
mod elem_compound_brace;
mod elem_compound_if;
mod elem_compound_while;
mod elem_end_of_command;
mod elem_end_of_list;
mod elem_function;
mod elem_script;
mod elem_command;
mod elem_redirect;
mod elem_setvars;
mod elem_subarg_braced;
mod elem_subarg_command_substitution;
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

use std::{env, process, path};
use std::os::linux::fs::MetadataExt;
use std::fs::{File,OpenOptions};
use std::io::{Read, BufRead, BufReader};

use crate::core::ShellCore;
use crate::feeder::Feeder;

use crate::abst_elems::{ListElem, PipelineElem};
use crate::elem_command::Command;
use crate::elem_script::Script;

fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match path::Path::new(&std_path).metadata() {
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

fn show_version() {
    const V: &'static str = env!("CARGO_PKG_VERSION");
    eprintln!("Rusty Bash, Version {}", V);
    eprintln!("© 2022 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
}

fn has_option(arg: &String, opt: String) -> bool {
    if arg.len() < 2 {
        return false;
    }

    if arg.chars().nth(0) == Some('-') && arg.chars().nth(1) == Some('-') { // --option
        return arg[2..] == opt;
    }

    if arg.chars().nth(0) == Some('-') { // -options
        return arg.chars().any(|c| c.to_string() == opt);
    }


    false
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    /*
    eprintln!("{}", glob_match(&args[1].to_string(), &args[2].to_string()));
    process::exit(0);
    */

    /* Ignore Ctrl+C (Childlen will receive instead.) */
    ctrlc::set_handler(move || { })
    .expect("Unable to set the Ctrl+C handler.");

    let mut core = ShellCore::new();
    for arg in &args {
        core.args.push(arg.clone());
    }

    core.flags.d = args.iter().any(|a| has_option(a, "d".to_string()));
    core.flags.v = args.iter().any(|a| has_option(a, "v".to_string()));
    core.flags.x = args.iter().any(|a| has_option(a, "x".to_string()));

    let pid = process::id();
    core.vars.insert("$".to_string(), pid.to_string());
    core.vars.insert("IFS".to_string(), " \t\n".to_string());
    core.vars.insert("HOSTNAME".to_string(), get_hostname());
    core.vars.insert("SHELL".to_string(), "rustybash".to_string());
    core.vars.insert("BASH".to_string(), core.args[0].clone());
    core.flags.i = is_interactive(pid);

    read_bashrc(&mut core);

    let mut feeder = Feeder::new();
    loop {
        if !feeder.feed_line(&mut core) {
            if core.flags.i {
                continue;
            }else{
                break;
            }
        }
        while let Some(mut e) = Script::parse(&mut feeder, &mut core, false){
            if feeder.len() != 0 && feeder.nth(0) == ')' {
                feeder.consume(feeder.len());
                eprintln!("Unknown phrase");
                core.vars.insert("?".to_string(), "2".to_string());
                break;
            }
            e.exec(&mut core);
        }
    }

    if let Ok(status) = core.get_var(&"?".to_string())
                        .to_string().parse::<i32>(){
        process::exit(status);
    }else{
        eprintln!("Shell internal error");
        process::exit(1);
    }

}
