//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod error;
mod feeder;
mod elements;
mod signal;
mod proc_ctrl;
mod utils;

use builtins::{option, parameter};
use std::{env, process};
use std::sync::atomic::Ordering::Relaxed;
use crate::core::{builtins, ShellCore};
use crate::elements::script::Script;
use crate::error::parse::ParseError;
use crate::feeder::Feeder;
use utils::{exit, file_check, arg};
use error::input::InputError;

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
    if ! core.db.flags.contains("i") {
        return;
    }

    let mut dir = core.db.get_param("CARGO_MANIFEST_DIR").unwrap_or(String::new());
    if dir == "" {
        dir = core.db.get_param("HOME").unwrap_or(String::new());
    }

    let rc_file = dir + "/.sushrc";

    if file_check::is_regular_file(&rc_file) {
        let _ = core.run_builtin(&mut vec![".".to_string(), rc_file], &mut vec![]);
    }
}

fn configure(args: &Vec<String>, core: &mut ShellCore) {
    core.configure();
    let mut parameters = vec![args[0].clone()];
    let mut options = vec![];

    for i in 1..args.len() {
        if args[i].starts_with("-") {
            options.push(args[i].clone());
        }else{
            core.script_name = args[i].clone();
            parameters = args[i..].to_vec();
            break;
        }
    }

    if let Err(e) = parameter::set_positions(core, &parameters) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }
    if let Err(e) = option::set_options(core, &options) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }
}

fn main() {
    let mut args = arg::dissolve_options(&env::args().collect());
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    let mut options = vec![];
    loop {
        if let Some(opt) = arg::consume_with_next_arg("-o", &mut args) {
            options.push(opt);
        }else{
            break;
        }
    }

    let mut core = ShellCore::new();
    let compat_bash = arg::consume_option("-b", &mut args);
    if compat_bash {
        core.compat_bash = true;
        core.db.flags += "b";
    }

    for opt in options {
        if let Err(e) = core.options.set(&opt, true) {
            e.print(&mut core);
            process::exit(2);
        }
    }

    let c_parts = arg::consume_with_subsequents("-c", &mut args);
    if c_parts.len() != 0 {
        core.configure_c_mode();

        run_and_exit_c_option(&args, &c_parts, &mut core);
    }

    configure(&args, &mut core);

    signal::run_signal_check(&mut core);

    if core.script_name == "-" {
        read_rc_file(&mut core);
    }
    main_loop(&mut core);
}

fn set_history(core: &mut ShellCore, s: &str) {
    //if core.read_stdin || core.history.is_empty() {
    if core.db.flags.contains('i') || core.history.is_empty() {
        return;
    }

    core.history[0] = s.trim_end().replace("\n", "↵ \0").to_string();
    if core.history[0].is_empty()
    || (core.history.len() > 1 && core.history[0] == core.history[1]) {
        core.history.remove(0);
    }
}

fn show_message() {
    const V: &'static str = env!("CARGO_PKG_VERSION");
    const P: &'static str = env!("CARGO_BUILD_PROFILE");
    eprintln!("Rusty Bash (a.k.a. Sushi shell), version {} - {}", V, P);
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new("");
    feeder.main_feeder = true;

    if core.script_name != "-" {
        core.db.flags.retain(|f| f != 'i');
        if let Err(e) = feeder.set_file(&core.script_name) {
            ParseError::Input(e).print(core);
            process::exit(2);
        }
    }

    if core.db.flags.contains('i') {
        show_message();
    }

    loop {
        if let Err(e) = core.jobtable_check_status() {
            e.print(core);
        }

        if core.db.flags.contains('i') && core.options.query("monitor") {
            core.jobtable_print_status_change();
        }

        match feeder.feed_line(core) {
            Ok(()) => {}, 
            Err(InputError::Interrupt) => {
                signal::input_interrupt_check(&mut feeder, core);
                signal::check_trap(core);
                continue;
            },
            _ => break,
        }

        parse_and_exec(&mut feeder, core, true);
    }
    core.write_history_to_file();
    exit::normal(core);
}

fn run_and_exit_c_option(args: &Vec<String>, c_parts: &Vec<String>, core: &mut ShellCore) {
    if c_parts.len() < 2 {
        println!("{}: -c: option requires an argument", &args[0]);
        process::exit(2);                
    }

    let parameters = if c_parts.len() > 2 {
        c_parts[2..].to_vec()
    }else{
        vec![args[0].clone()]
    };

    if let Err(e) = parameter::set_positions(core, &parameters) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }
    if let Err(e) = option::set_options(core, &mut args[1..].to_vec()) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }

    signal::run_signal_check(core);
    core.db.flags.retain(|f| f != 'i');

    core.db.flags += "c";
    if core.db.flags.contains('v') {
        eprintln!("{}", &c_parts[1]);
    }

    let mut feeder = Feeder::new_c_mode(c_parts[1].clone());
    feeder.main_feeder = true;

    loop {
        if let Err(e) = core.jobtable_check_status() {
            e.print(core);
        }

        if core.db.flags.contains('i') && core.options.query("monitor") {
            core.jobtable_print_status_change();
        }

        match feeder.feed_line(core) {
            Ok(()) => {}, 
            Err(InputError::Interrupt) => {
                signal::input_interrupt_check(&mut feeder, core);
                signal::check_trap(core);
                continue;
            },
            _ => break,
        }

        parse_and_exec(&mut feeder, core, false);
    }
    exit::normal(core);
}

fn parse_and_exec(feeder: &mut Feeder, core: &mut ShellCore, set_hist: bool) {
    core.sigint.store(false, Relaxed);
    match Script::parse(feeder, core, false){
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
            if set_hist {
                set_history(core, &s.get_text());
            }
        },
        Err(e) => {
            e.print(core);
            feeder.consume(feeder.len());
            feeder.nest = vec![("".to_string(), vec![])];
        },
        _ => {
            feeder.consume(feeder.len());
            feeder.nest = vec![("".to_string(), vec![])];
        },
    }
    core.sigint.store(false, Relaxed);
}
