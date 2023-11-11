//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process};
use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::Feeder;

use std::sync::Arc;
use std::{thread, time};
use signal_hook::consts::SIGCHLD;
use signal_hook::iterator::Signals;
use crate::core::jobtable::JobEntry;
use std::sync::Mutex;
use nix::unistd;

fn show_version() {
    eprintln!("Sushi Shell 202305_5");
    eprintln!("© 2023 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
}

fn check_signal(signal: i32, jt: &Arc<Mutex<Vec<JobEntry>>>) {
    match signal {
        SIGCHLD => {
            let mut mtx = jt.lock().unwrap();
            for e in mtx.iter_mut() {
                e.update_status();
            }
        },
        _ => {}, 
    }
}

//thanks: https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6 
fn run_childcare(core: &mut ShellCore) {
    for fd in 3..10 { //use FD 3~9 to prevent signal-hool from using these FDs
        unistd::dup2(2, fd).expect("sush(fatal): init error");
    }

    let jt = Arc::clone(&core.job_table); 
    thread::spawn(move || { 
        let mut signals = Signals::new(vec![SIGCHLD])
                          .expect("sush(fatal): cannot prepare signal data");

        for fd in 3..10 { // release FD 3~9
            unistd::close(fd).expect("sush(fatal): init error");
        }

        loop {
            thread::sleep(time::Duration::from_secs(1));
            for signal in signals.pending() {
                check_signal(signal, &jt);
            }
        }
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }
    let mut core = ShellCore::new();

    run_childcare(&mut core);

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
