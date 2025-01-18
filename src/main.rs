//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod error;
mod feeder;
mod elements;
mod signal;
mod utils;

use std::{env, process};
use crate::core::ShellCore;
use crate::utils::exit;
use crate::elements::script::Script;
use crate::error::exec;
use crate::error::input::InputError;
use crate::feeder::Feeder;
use utils::file_check;
use std::sync::atomic::Ordering::Relaxed;

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

    let mut core = ShellCore::new();
    signal::run_signal_check(&mut core);
    main_loop(&mut core);
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
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

        if let Ok(Some(mut s)) = Script::parse(&mut feeder, core){
            if let Err(e) = s.exec(core) {
                exec::print_error(e, core);
            }
        }
        core.sigint.store(false, Relaxed);
    }

    exit::normal(core)
}
