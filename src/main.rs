//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod error;
mod feeder;
mod elements;
mod main_c_option;
mod signal;
mod proc_ctrl;
mod utils;

// Externals crates
use std::{env, fs, process};
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;
use fluent_bundle::{FluentBundle, FluentResource};
use once_cell::unsync::OnceCell;
use unic_langid::LanguageIdentifier;

// Internals crates
use crate::core::{builtins, ShellCore};
use crate::core::builtins::source;
use crate::elements::script::Script;
use crate::feeder::Feeder;
use error::input::InputError;
use utils::{arg, exit, file_check};
use builtins::option;

thread_local! {
    static FLUENT_BUNDLE: RefCell<OnceCell<FluentBundle<FluentResource>>> = RefCell::new(OnceCell::new());
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
        core.db.exit_status = source::source(core, &mut vec![".".to_string(), rc_file]);
    }
}

fn consume_file_and_subsequents(args: &mut Vec<String>) -> Vec<String> {
    let len = args.len();
    let mut skip = false;
    let mut pos = None;

    for i in 1..len {
        if skip {
            skip = false;
            continue;
        }

        if args[i].starts_with("-o") || args[i].starts_with("+o") {
            skip = true;
            continue;
        }

        if args[i].starts_with("-") || args[i].starts_with("+") {
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

fn set_parameters(script_parts: Vec<String>, core: &mut ShellCore, command: &str) {
    match script_parts.is_empty() {
        true  => {
            core.db.position_parameters[0] = vec![command.to_string()];
            core.script_name = "-".to_string();
        },
        false => {
            core.db.position_parameters[0] = script_parts;
            core.script_name = core.db.position_parameters[0][0].clone();
        },
    }
}

fn main() {
    let mut args = arg::dissolve_options_main();

    if args.iter().any(|arg| arg == "--version") {
        show_version();
    }

    let command = args.get(0).cloned().unwrap_or_else(|| "sush".to_string());
    let script_parts = consume_file_and_subsequents(&mut args);

    let mut c_opt = false;
    if let Some(opt) = args.last() {
        if opt == "-c" {
            c_opt = true;
            args.pop();
        }
    }

    let mut core = ShellCore::new();
    set_o_options(&mut args, &mut core);
    set_short_options(&mut args, &mut core);

    if ! c_opt {
        set_parameters(script_parts, &mut core, &command);
    }else{
        main_c_option::set_parameters(&script_parts, &mut core, &args[0]);
        main_c_option::run_and_exit(&args, &script_parts, &mut core); //exit here
    }

    core.configure();
    signal::run_signal_check(&mut core);

    if core.script_name == "-" {
        read_rc_file(&mut core);
    }
    main_loop(&mut core, &command);
}

fn set_history(core: &mut ShellCore, s: &str) {
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

fn main_loop(core: &mut ShellCore, command: &String) {
    let mut feeder = Feeder::new("");
    feeder.main_feeder = true;

    if core.script_name != "-" {
        core.db.flags.retain(|f| f != 'i');
        if let Err(_) = feeder.set_file(&core.script_name) {
            eprintln!("{}: {}: No such file or directory", command, &core.script_name); 
            process::exit(2);
        }
    }

    if core.db.flags.contains('i') {
        show_message();
    }

    loop {
        match feed_script(&mut feeder, core) {
            (true, false) => {},
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
    if let Err(e) = core.jobtable_check_status() {          //(continue, break)
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
        },
        _ => (false, true),
    }
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

fn get_system_language() -> String {
    fn extract_lang(s: &str) -> Option<String> {
        let first_part = s.split(&['_', '.']).next()?;
        if first_part.is_empty() {
            None
        } else {
            Some(first_part.to_string())
        }
    }

    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if let Some(lang) = extract_lang(&val) {
                return lang;
            }
        }
    }

    "en".to_string()
}

fn load_fluent_bundle(lang: &str) -> FluentBundle<FluentResource> {
    let langid: LanguageIdentifier = lang.parse().unwrap_or_else(|_| "en".parse().unwrap());
    let ftl_string = fs::read_to_string(format!("i18n/{}.ftl", lang))
        .or_else(|_| fs::read_to_string("i18n/en.ftl"))
        .expect("No suitable .ftl file found");
    let resource = FluentResource::try_new(ftl_string).expect("Invalid FTL syntax");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(resource).expect("Failed to add resource");
    bundle
}

fn fl(key: &str) -> String {
    FLUENT_BUNDLE.with(|cell| {
        let cell = cell.borrow_mut();
        let bundle = cell.get_or_init(|| {
            let lang = get_system_language();
            load_fluent_bundle(&lang)
        });

        let mut errors = vec![];
        bundle
            .get_message(key)
            .and_then(|msg| msg.value())
            .map(|pattern| bundle.format_pattern(pattern, None, &mut errors).to_string())
            .unwrap_or_else(|| format!("{{{}}}", key))
    })
}

fn show_version() {
    const V: &'static str = env!("CARGO_PKG_VERSION");
    const P: &'static str = env!("CARGO_BUILD_PROFILE");
    eprintln!(
        "Rusty Bash (a.k.a. Sushi shell), version {} - {}\n\
         © 2024 Ryuichi Ueda\n\
         {}: BSD 3-Clause\n\
         \n\
         {}",
        V, P, fl("license"), fl("text-version")
    );
    process::exit(0);
}
