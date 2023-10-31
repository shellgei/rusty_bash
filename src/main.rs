//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process};
use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::Feeder;
use nix::sys::signal;
use nix::sys::signal::{Signal, SigHandler};

fn show_version() {
    eprintln!("Sushi Shell 202305_5");
    eprintln!("© 2023 Ryuichi Ueda");
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

    unsafe { signal::signal(Signal::SIGTTIN, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGTTOU, SigHandler::SigIgn) }.unwrap();

    let mut core = ShellCore::new();
    main_loop(&mut core);
}

fn input_interrupt_check(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    if ! core.input_interrupt {
        return false;
    }

    core.input_interrupt = false;
    core.vars.insert("?".to_string(), "130".to_string());
    feeder.consume(feeder.len());
    true
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        core.jobtable_check();
        if ! feeder.feed_line(core) {
            if core.has_flag('i') {
                input_interrupt_check(&mut feeder, core);
                continue;
            }else{
                break;
            }   
        }

        match Script::parse(&mut feeder, core){
            Some(mut s) => {
                if ! input_interrupt_check(&mut feeder, core) {
                    s.exec(core, &mut vec![])
                }
            },
            None => continue,
        }
    }
    core.exit();
}
