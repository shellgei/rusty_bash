//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;
mod signal;
mod utils;

use builtins::option_commands;
use std::{env, process};
use std::sync::atomic::Ordering::Relaxed;
use crate::core::{builtins, ShellCore};
use crate::elements::script::Script;
use crate::feeder::{Feeder, InputError};
use utils::{exit, file_check};

fn show_version() {
    const V: &'static str = env!("CARGO_PKG_VERSION");
    const P: &'static str = env!("CARGO_BUILD_PROFILE");
    eprintln!("Rusty Bash (a.k.a. Sushi shell), version {} - {}
© 2024 Ryuichi Ueda
License: BSD 3-Clause

This is open source software. You can redistirbute and use in source
and binary forms with or without modification under the license.
There is no warranty, to the extent permitted by law.", V, P);
    process::exit(0);
}

fn read_rc_file(core: &mut ShellCore) {
    if ! core.data.flags.contains("i") {
        return;
    }

    let dir = match core.data.get_param("CARGO_MANIFEST_DIR").as_str() {
        "" => core.data.get_param("HOME"),
        s  => s.to_string(),
    };

    let rc_file = dir + "/.sushrc";

    if file_check::is_regular_file(&rc_file) {
        core.run_builtin(&mut vec![".".to_string(), rc_file], &mut vec![]);
    }
}

fn configure(args: &Vec<String>, options: &mut Vec<String>, parameters: &mut Vec<String>,
             script: &mut String, c_flag: &mut bool) {
    for i in 1..args.len() {
        if args[i] == "-c" {
            *c_flag = true;
            if i == args.len()-1 {
                eprintln!("bash: -c: option requires an argument");
                process::exit(2);
            }
            *script = args[i+1].to_string();
            break;
        }

        if args[i].starts_with("-") {
            parameters.remove(i);
            options.push(args[i].clone());
        }else{
            *script = args[i].clone();
            *parameters = args[i..].to_vec();
            break;
        }
    }
}

/*
fn set_script_file(script: &str) {
    match File::open(script) {
        Ok(file) => {
            let fd = file.into_raw_fd();
            let result = io::replace(fd, 0);
            if ! result {
                io::close(fd, &format!("sush(fatal): file does not close"));
            }
        },
        Err(why)  => {
            eprintln!("sush: {}: {}", script, why);
            process::exit(1);
        },
    }
}*/

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    let mut options = args[0..1].to_vec();
    let mut parameters = args.to_vec();
    let mut script = "-".to_string();
    let mut c_flag = false;

    configure(&args, &mut options, &mut parameters, &mut script, &mut c_flag);

    /*
    if script != "-" && ! c_flag {
        set_script_file(&script);
    }*/

    let mut core = ShellCore::new();
    option_commands::set(&mut core, &mut options);
    option_commands::set_parameters(&mut core, &mut parameters);
    signal::run_signal_check(&mut core);

    if c_flag {
        main_c_option(&mut core, &script);
        exit::normal(&mut core);
    }

    if script == "-" {
        read_rc_file(&mut core);
    }
    core.script_name = script.clone();
    main_loop(&mut core);
}

fn set_history(core: &mut ShellCore, s: &str) {
    if core.read_stdin || core.history.len() == 0 {
        return;
    }

    core.history[0] = s.trim_end().replace("\n", "↵ \0").to_string();
    if core.history[0].len() == 0
    || (core.history.len() > 1 && core.history[0] == core.history[1]) {
        core.history.remove(0);
    }
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new("");

    if core.script_name != "-" {
        //let file = File::open(core.script_name.clone());
        feeder.set_file(&core.script_name);
    }

    loop {
        core.jobtable_check_status();
        core.jobtable_print_status_change();

        match feeder.feed_line(core) {
            Ok(()) => {}, 
            Err(InputError::Interrupt) => {
                signal::input_interrupt_check(&mut feeder, core);
                continue;
            },
            _ => break,
        }

        core.word_eval_error = false;
        core.sigint.store(false, Relaxed);
        match Script::parse(&mut feeder, core, false){
            Some(mut s) => {
                s.exec(core);
                set_history(core, &s.get_text());
            },
            None => {},
        }
        core.sigint.store(false, Relaxed);
    }
    core.write_history_to_file();
    exit::normal(core);
}

fn main_c_option(core: &mut ShellCore, script: &String) {
    core.data.flags += "c";
    let mut feeder = Feeder::new(script);
    if let Some(mut s) = Script::parse(&mut feeder, core, false){
        s.exec(core);
    }
    exit::normal(core)
}
