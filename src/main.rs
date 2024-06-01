//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process, thread, time};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use crate::core::{builtins, ShellCore};
use crate::elements::script::Script;
use crate::feeder::{Feeder, InputError};
use signal_hook::consts;
use signal_hook::iterator::Signals;

fn show_version() {
    const V: &'static str = env!("CARGO_PKG_VERSION");
    eprintln!("Rusty Bash (a.k.a. Sushi shell), version {}
© 2024 Ryuichi Ueda
License: BSD 3-Clause

This is open source software. You can redistirbute and use in source
and binary forms with or without modification under the license.
There is no warranty, to the extent permitted by law.", V);
    process::exit(0);
}

fn run_signal_check(core: &mut ShellCore) {
    for fd in 3..10 { //use FD 3~9 to prevent signal-hool from using these FDs
        nix::unistd::dup2(2, fd).expect("sush(fatal): init error");
    }

    let sigint = Arc::clone(&core.sigint); //追加
 
    thread::spawn(move || {
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

fn read_rc_file(core: &mut ShellCore) {
    let dir = match core.data.get_param("CARGO_MANIFEST_DIR").as_str() {
        "" => core.data.get_param("HOME"),
        s  => s.to_string(),
    };

    let rc_file = dir + "/.sushrc";

    if Path::new(&rc_file).is_file() {
        core.run_builtin(&mut vec![".".to_string(), rc_file], &mut vec![]);
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    let mut core = ShellCore::new();
    builtins::set::set_parameters(&mut core, &mut args);
    run_signal_check(&mut core);
    read_rc_file(&mut core);
    main_loop(&mut core);
}

fn set_history(core: &mut ShellCore, s: &str) {
    if core.read_stdin || core.history.len() == 0 {
        return;
    }

    core.history[0] = s.trim_end().to_string();
    if core.history[0].len() == 0
    || (core.history.len() > 1 && core.history[0] == core.history[1]) {
        core.history.remove(0);
    }
}

fn input_interrupt_check(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    if ! core.sigint.load(Relaxed) { //core.input_interrupt {
        return false;
    }

    core.sigint.store(false, Relaxed); //core.input_interrupt = false;
    core.data.set_param("?", "130");
    feeder.consume(feeder.len());
    true
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new("");
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
        match Script::parse(&mut feeder, core, false){
            Some(mut s) => {
                s.exec(core);
                set_history(core, &s.get_text());
            },
            None => {},
        }
        core.sigint.store(false, Relaxed);
    }
    core.exit();
}
