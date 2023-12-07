//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process, thread, time};
use std::sync::{Arc, Mutex};
use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::Feeder;
use signal_hook::consts;
use signal_hook::iterator::Signals;

fn show_version() {
    eprintln!("Sushi Shell 202305_5");
    eprintln!("© 2023 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
}

//thanks: https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6
fn run_signal_check(core: &mut ShellCore) {
    let mut flags = Arc::clone(&core.signal_flags);
    thread::spawn(move || {
        let mut signals = Signals::new(vec![consts::SIGINT])
                          .expect("sush(fatal): cannot prepare signal data");

        loop {
            thread::sleep(time::Duration::from_millis(100));
            for signal in signals.pending() {
                check_signals(signal, &mut flags);
            }
        }
    });
}

fn check_signals(signal: i32, flags: &mut Arc<Mutex<Vec<bool>>>) {
    match signal {
        consts::SIGINT => {
            let mut mtx = flags.lock().unwrap();
            mtx[consts::SIGINT as usize] = true;
        },
        _ => {},
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    let mut core = ShellCore::new();
    run_signal_check(&mut core);
    main_loop(&mut core);
}

fn input_interrupt_check(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    if ! core.check_signal(consts::SIGINT) {
        return false;
    }

    core.unset_signal(consts::SIGINT);
    core.vars.insert("?".to_string(), "130".to_string());
    feeder.consume(feeder.len());
    true
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        core.jobtable_check_status();
        core.jobtable_print_status_change();
        if ! feeder.feed_line(core) {
            if core.has_flag('i') {
                input_interrupt_check(&mut feeder, core);
                continue;
            }else{
                break;
            }   
        }

        match Script::parse(&mut feeder, core){
            Some(mut s) => s.exec(core),
            None => continue,
        }
    }
    core.exit();
}
