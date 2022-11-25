//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod abst_elems;
mod bash_glob;
mod calculator;
mod elements;
mod element_list;

mod core;
mod job;
mod term;
mod file_descs;
mod utils;
mod feeder;
mod debuginfo;

use std::{env, process, path};
use std::os::linux::fs::MetadataExt;
use std::fs::{File,OpenOptions};
use std::io::Read;

use crate::core::ShellCore;
use crate::feeder::Feeder;

use crate::abst_elems::ListElem;
use crate::elements::script::Script;

use crate::element_list::CompoundType;

use crate::file_descs::FileDescs;
use std::os::unix::io::IntoRawFd;

fn is_interactive(pid: u32) -> bool {
    let std_path = format!("/proc/{}/fd/0", pid);
    match path::Path::new(&std_path).metadata() {
        Ok(metadata) => metadata.st_mode() == 8592, 
        Err(err) => panic!("{}", err),
    }
}

fn read_bashrc(core: &mut ShellCore){
    let home = env::var("HOME").expect("HOME is not defined");
    if let Ok(_) = File::open(home.clone() + "/.rusty_bashrc") {
        let f = core.builtins["source"];
        let mut args = vec!("source".to_string(), home.clone() + "/.rusty_bashrc");
        f(core, &mut args);
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

    /* Ignore Ctrl+C (Childlen will receive instead.) */
    ctrlc::set_handler(move || { })
    .expect("Unable to set the Ctrl+C handler.");

    let mut core = ShellCore::new();
    for arg in &args {
        core.args.push(arg.clone());
    }

    if args.len() > 1 {
        if let Ok(file) = OpenOptions::new().read(true).open(&args[1]){
            FileDescs::dup_and_close(file.into_raw_fd(), 0);
        }
    }

    for f in [ "d", "v", "x" ] {
        if args.iter().any(|a| has_option(a, f.to_string())) {
            core.flags += f;
        }
    }

    let pid = process::id();
    core.set_var("$", &pid.to_string());
    core.set_var("IFS", " \t\n");
    core.set_var("HOSTNAME", &get_hostname());
    core.set_var("SHELL", "rustybash");
    core.set_var("BASH", &core.args[0].to_string());
    if is_interactive(pid) {
        core.flags += "i";
    }

    read_bashrc(&mut core);
    main_loop(&mut core);
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        if !feeder.feed_line(core) {
            if core.has_flag('i') {
                continue;
            }else{
                break;
            }
        }
        while let Some(mut e) = Script::parse(&mut feeder, core, &CompoundType::Null){
            if feeder.len() != 0 && feeder.nth(0) == ')' {
                feeder.consume(feeder.len());
                eprintln!("Unknown phrase");
                core.set_var("?", "2");
                break;
            }
            e.exec(core);
        }
    }

    //if let Ok(status) = core.get_var(&"?".to_string())
    if let Ok(status) = core.get_var("?")
                        .to_string().parse::<i32>(){
        process::exit(status);
    }else{
        eprintln!("Shell internal error");
        process::exit(1);
    }
}
