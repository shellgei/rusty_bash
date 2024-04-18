//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process, thread, time};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::{Feeder, InputError};
use signal_hook::consts;
use signal_hook::iterator::Signals;

fn show_version() {
    let s = "Sushi Shell SoftwareDesign version
© 2023 Ryuichi Ueda
License: BSD 3-Clause

This is open source software. You can redistirbute and use in source
and binary forms with or without modification under the license.
There is no warranty, to the extent permitted by law.";
    eprintln!("{}", s);
    process::exit(0);
}

fn run_signal_check(core: &mut ShellCore) {
    for fd in 3..10 { //use FD 3~9 to prevent signal-hool from using these FDs
        nix::unistd::dup2(2, fd).expect("sush(fatal): init error");
    }

    let sigint = Arc::clone(&core.sigint); //追加
 
    thread::spawn(move || { //クロージャの処理全体を{}で囲みましょう
        let mut signals = Signals::new(vec![consts::SIGINT])
                          .expect("sush(fatal): cannot prepare signal data");

        for fd in 3..10 { // release FD 3~9
            nix::unistd::close(fd).expect("sush(fatal): init error");
        }

        loop {
            thread::sleep(time::Duration::from_millis(100)); //0.1秒周期に変更
            for signal in signals.pending() {
                if signal == consts::SIGINT {
                    sigint.store(true, Relaxed);
                    //eprint!("COME HERE");
                }
            }
        }
    });
} //thanks: https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6

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
    if ! core.sigint.load(Relaxed) { //core.input_interrupt {
        return false;
    }

    core.sigint.store(false, Relaxed); //core.input_interrupt = false;
    core.set_param("?", "130");
    feeder.consume(feeder.len());
    true
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        core.jobtable_check_status();
        core.jobtable_print_status_change();

        match feeder.feed_line(core) {
            Ok(()) => {}, 
            Err(InputError::Interrupt) => {
                input_interrupt_check(&mut feeder, core);
                continue;
            },
            _ => break,
        }

        core.sigint.store(false, Relaxed);
        match Script::parse(&mut feeder, core){
            Some(mut s) => s.exec(core),
            None => {},
        }
        core.sigint.store(false, Relaxed);
    }
    core.exit();
}
