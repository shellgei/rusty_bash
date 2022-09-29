//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod bash_glob;
mod core;
mod core_builtins;
mod core_shopts;
mod term;
mod utils;
mod utils_io;
mod feeder;
mod scanner;
mod debuginfo;
mod term_completion;

use std::{env, process};

use crate::core::ShellCore;
use crate::feeder::Feeder;

/*
fn get_hostname() -> String{
    if let Ok(mut file) = File::open("/etc/hostname") {

        let mut fullname = String::new();
        if let Ok(_) = file.read_to_string(&mut fullname) {
            return fullname.trim_end().to_string();
        }
    }

    "unknown".to_string()
}*/

fn show_version() {
    eprintln!("Rusty Bash, TERMINAL SKELETON");
    eprintln!("© 2022 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
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
    /*
    for arg in &args {
        core.args.push(arg.clone());
    }*/

    /*
    if args.len() > 1 {
        if let Ok(file) = OpenOptions::new().read(true).open(&args[1]){
            dup_and_close(file.into_raw_fd(), 0);
        }
    }
    */

    /*
    for f in [ "d", "v", "x" ] {
        if args.iter().any(|a| has_option(a, f.to_string())) {
            core.flags += f;
        }
    }*/

    /*
    let pid = process::id();
    core.set_var("$", &pid.to_string());
    core.set_var("IFS", " \t\n");
    core.set_var("HOSTNAME", &get_hostname());
    core.set_var("SHELL", "rustybash");
    core.set_var("BASH", &core.args[0].to_string());
    */

    main_loop(&mut core);
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        if feeder.feed_line(core) {
        }
    }
}
