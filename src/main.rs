//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod bash_glob;
mod calculator;
mod elements;
mod operators;

mod core;
mod file_descs;
mod utils;
mod feeder;
mod debuginfo;

use std::{env, process};
use std::fs::{File,OpenOptions};
use nix::libc;
use nix::unistd::isatty;
use nix::sys::utsname::uname;

use crate::core::ShellCore;
use crate::core::proc;
use crate::feeder::Feeder;

use crate::elements::script::Script;

use crate::file_descs::FileDescs;
use std::os::unix::io::IntoRawFd;

fn is_interactive() -> bool {
    match isatty(libc::STDIN_FILENO) {
        Ok(atty) => atty,
        Err(err) => panic!("{}", err),
    }
}

fn read_bashrc(core: &mut ShellCore){
    let home = env::var("HOME").expect("HOME is not defined");
    if let Ok(_) = File::open(home.clone() + "/.rusty_bashrc") {
        let f = core.builtins["source"];
        let mut words = vec!("source".to_string(), home.clone() + "/.rusty_bashrc");
        f(core, &mut words);
    }
}

fn get_hostname() -> String{
    if let Ok(uts) = uname() {
        let fullname = uts.nodename().to_string_lossy();
        return fullname.to_string();
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

fn has_option(word: &String, opt: String) -> bool {
    if word.len() < 2 {
        return false;
    }

    if word.chars().nth(0) == Some('-') && word.chars().nth(1) == Some('-') { // --option
        return word[2..] == opt;
    }

    if word.chars().nth(0) == Some('-') { // -options
        return word.chars().any(|c| c.to_string() == opt);
    }


    false
}

fn main() {
    let words: Vec<String> = env::args().collect();
    if words.len() > 1 && words[1] == "--version" {
        show_version();
    }

    /* Ignore signals */
    proc::ignore_signals();
    /*
    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGTTIN, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGTTOU, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGTSTP, SigHandler::SigIgn) }.unwrap();
    */

    let mut core = ShellCore::new();
    for word in &words {
        core.args.push(word.clone());
    }

    if words.len() > 1 {
        if let Ok(file) = OpenOptions::new().read(true).open(&words[1]){
            FileDescs::dup_and_close(file.into_raw_fd(), 0);
        }
    }

    for f in [ "d", "v", "x" ] {
        if words.iter().any(|a| has_option(a, f.to_string())) {
            core.flags += f;
        }
    }

    let pid = process::id();
    core.set_var("$", &pid.to_string());
    core.set_var("IFS", " \t\n");
    core.set_var("HOSTNAME", &get_hostname());
    core.set_var("SHELL", "rustybash");
    core.set_var("BASH", &core.args[0].to_string());
    if is_interactive() {
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
        while let Some(mut e) = Script::parse(&mut feeder, core){
//            eprintln!("{:?}", &e);
            if feeder.len() != 0 && feeder.nth(0) == ')' {
                feeder.consume(feeder.len());
                eprintln!("Unknown phrase");
                core.set_var("?", "2");
                break;
            }
            e.exec(core);
        }
        core.check_jobs();
    }

    if let Ok(status) = core.get_var("?").to_string().parse::<i32>(){
        process::exit(status);
    }else{
        eprintln!("Shell internal error");
        process::exit(1);
    }
}
