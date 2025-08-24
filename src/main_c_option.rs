//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::{builtins, ShellCore};
use crate::feed_script;
use crate::feeder::Feeder;
use crate::parse_and_exec;
use crate::signal;
use crate::utils::exit;
use builtins::option;
use std::process;

pub fn set_parameters(c_parts: &[String], core: &mut ShellCore, command: &str) {
    let parameters = if c_parts.len() > 1 {
        c_parts[1..].to_vec()
    } else {
        vec![command.to_string()]
    };

    if let Err(e) = option::set_positions_c(core, &parameters) {
        e.print(core);
        core.db.exit_status = 2;
        exit::normal(core);
    }
}

pub fn run_and_exit(args: &[String], c_parts: &[String], core: &mut ShellCore) {
    core.configure_c_mode();

    if c_parts.is_empty() {
        println!("{}: -c: option requires an argument", &args[0]);
        process::exit(2);
    }

    signal::run_signal_check(core);
    core.db.flags.retain(|f| f != 'i');

    core.db.flags += "c";
    if core.db.flags.contains('v') {
        eprintln!("{}", &c_parts[0]);
    }

    let mut feeder = Feeder::new_c_mode(c_parts[0].clone());
    feeder.main_feeder = true;

    loop {
        match feed_script(&mut feeder, core) {
            (true, false) => {}
            (false, true) => break,
            _ => parse_and_exec(&mut feeder, core, false),
        }
    }
    exit::normal(core);
}
