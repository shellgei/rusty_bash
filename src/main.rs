//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod elements;
mod error;
mod feeder;
mod i18n;
mod main_c_option;
mod proc_ctrl;
mod signal;
mod utils;

// Externals crates
use std::sync::atomic::Ordering::Relaxed;
use std::{env, process};

// Internals crates
use crate::core::builtins::source;
use crate::core::{builtins, ShellCore};
use crate::elements::script::Script;
use crate::feeder::Feeder;
use crate::i18n::FLUENT_BUNDLE;
use builtins::option;
use error::input::InputError;
use utils::{arg, exit, file_check};

///// Main program entry point /////

fn main() {
    let mut args = arg::dissolve_options_main();

    if args.iter().any(|a| a == "--version") {
        show_version();
        return;
    }

    if args.iter().any(|a| a == "--help") {
        show_help();
        return;
    }

    let command = args.first().cloned().unwrap_or_else(|| "sush".to_string());
    let script_parts = consume_file_and_subsequents(&mut args);

    let mut c_opt = false;
    if let Some(opt) = args.last() {
        if opt == "-c" {
            c_opt = true;
            args.pop();
        }
    }

    let mut core = ShellCore::new();

    let bundle = match i18n::load_fluent_bundle() {
        Some(b) => b,
        None => {
            eprintln!("No resources found for language");
            std::process::exit(1);
        }
    };

    FLUENT_BUNDLE.with(|cell| {
        cell.set(bundle).ok();
    });

    set_o_options(&mut args, &mut core);
    set_short_options(&mut args, &mut core);

    if !c_opt {
        set_parameters(script_parts, &mut core, &command);
    } else {
        main_c_option::set_parameters(&script_parts, &mut core, &args[0]);
        main_c_option::run_and_exit(&args, &script_parts, &mut core);
    }

    core.configure();
    signal::run_signal_check(&mut core);

    if core.script_name == "-" {
        read_rc_file(&mut core);
    }
    main_loop(&mut core, &command);
}

///// Parses arguments and sets up shell options, parameters, and config. /////

fn consume_file_and_subsequents(args: &mut Vec<String>) -> Vec<String> {
    let mut skip = false;
    let mut pos = None;

    for (i, a) in args.iter().enumerate().skip(1) {
        if skip {
            skip = false;
            continue;
        }

        if a.starts_with("-o") || a.starts_with("+o") {
            skip = true;
            continue;
        }

        if a.starts_with('-') || a.starts_with('+') {
            continue;
        }

        pos = Some(i);
        break;
    }

    if pos.is_none() {
        return vec![];
    }

    args.split_off(pos.unwrap())
}

fn set_o_options(args: &mut Vec<String>, core: &mut ShellCore) {
    let mut options = vec![];
    loop {
        if let Some(opt) = arg::consume_with_next_arg("-o", args) {
            options.push((opt, true));
            continue;
        }
        if let Some(opt) = arg::consume_with_next_arg("+o", args) {
            options.push((opt, false));
            continue;
        }

        break;
    }

    for opt in options {
        if let Err(e) = core.options.set(&opt.0, opt.1) {
            e.print(core);
            process::exit(2);
        }
    }
}

fn set_short_options(args: &mut Vec<String>, core: &mut ShellCore) {
    if arg::consume_option("-b", args) {
        core.compat_bash = true;
        core.db.flags += "b";
    }

    if let Err(e) = option::set_options(core, &mut args[1..].to_vec()) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }
}

fn read_rc_file(core: &mut ShellCore) {
    if !core.db.flags.contains("i") {
        return;
    }

    let mut dir = core.db.get_param("CARGO_MANIFEST_DIR").unwrap_or_default();
    if dir.is_empty() {
        dir = core.db.get_param("HOME").unwrap_or_default();
    }

    let rc_file = dir + "/.sushrc";

    if file_check::is_regular_file(&rc_file) {
        core.db.exit_status = source::source(core, &[".".to_string(), rc_file]);
    }
}

fn set_parameters(script_parts: Vec<String>, core: &mut ShellCore, command: &str) {
    match script_parts.is_empty() {
        true => {
            core.db.position_parameters[0] = vec![command.to_string()];
            core.script_name = "-".to_string();
        }
        false => {
            core.db.position_parameters[0] = script_parts;
            core.script_name = core.db.position_parameters[0][0].clone();
        }
    }
}

///// Core shell flow: input, parsing, execution, and history. /////

fn main_loop(core: &mut ShellCore, command: &str) {
    let mut feeder = Feeder::new("");
    feeder.main_feeder = true;

    if core.script_name != "-" {
        core.db.flags.retain(|f| f != 'i');
        if feeder.set_file(&core.script_name).is_err() {
            eprintln!(
                "{}: {}: No such file or directory",
                command, &core.script_name
            );
            process::exit(2);
        }
    }

    if core.db.flags.contains('i') {
        show_message();
    }

    loop {
        match feed_script(&mut feeder, core) {
            (true, false) => {}
            (false, true) => break,
            _ => parse_and_exec(&mut feeder, core, true),
        }

        if core.options.query("onecmd") {
            break;
        }
    }
    core.write_history_to_file();
    exit::normal(core);
}

fn feed_script(feeder: &mut Feeder, core: &mut ShellCore) -> (bool, bool) {
    if let Err(e) = core.jobtable_check_status() {
        //(continue, break)
        e.print(core);
    }

    if core.db.flags.contains('i') && core.options.query("monitor") {
        core.jobtable_print_status_change();
    }

    match feeder.feed_line(core) {
        Ok(()) => (false, false),
        Err(InputError::Interrupt) => {
            signal::input_interrupt_check(feeder, core);
            signal::check_trap(core);
            (true, false)
        }
        _ => (false, true),
    }
}

fn parse_and_exec(feeder: &mut Feeder, core: &mut ShellCore, set_hist: bool) {
    core.sigint.store(false, Relaxed);
    match Script::parse(feeder, core, false) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
            if set_hist {
                set_history(core, &s.get_text());
            }
        }
        Err(e) => {
            e.print(core);
            feeder.consume(feeder.len());
            feeder.nest = vec![("".to_string(), vec![])];
        }
        _ => {
            feeder.consume(feeder.len());
            feeder.nest = vec![("".to_string(), vec![])];
        }
    }
    core.sigint.store(false, Relaxed);
}

fn set_history(core: &mut ShellCore, s: &str) {
    if core.db.flags.contains('i') || core.history.is_empty() {
        return;
    }

    core.history[0] = s.trim_end().replace("\n", "↵ \0").to_string();
    if core.history[0].is_empty() || (core.history.len() > 1 && core.history[0] == core.history[1])
    {
        core.history.remove(0);
    }
}

///// Text related functions /////

fn show_message() {
    eprintln!(
        "Sushi shell (a.k.a. Sush), {} {} - {}",
        i18n::fl("version"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_BUILD_PROFILE")
    );
}

fn show_version() {
    eprintln!(
        "Sushi shell (a.k.a. Sush), {} {} - {}\n\
         © 2024 Ryuichi Ueda\n\
         {}: BSD 3-Clause\n\
         \n\
         {}\n",
        i18n::fl("version"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_BUILD_PROFILE"),
        i18n::fl("license"),
        i18n::fl("text-version")
    );
    process::exit(0);
}

fn show_help() {
    eprintln!(
        "Sushi shell (a.k.a. Sush), {} {} - {}\n\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n\n{}",
        i18n::fl("version"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_BUILD_PROFILE"),
        i18n::fl("usage"),
        i18n::fl("options"),
        i18n::fl("comp-commands"),
        i18n::fl("builtins"),
        i18n::fl("parameters"),
        i18n::fl("shopt"),
        i18n::fl("variables-born"),
        i18n::fl("variables-bash"),
        i18n::fl("text-help")
    );
    process::exit(0);
}
