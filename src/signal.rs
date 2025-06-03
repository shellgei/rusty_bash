//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::sys::signal;
use nix::sys::signal::{Signal, SigHandler};
use std::{thread, time};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use crate::Script;
use crate::core::ShellCore;
use crate::feeder::Feeder;
use signal_hook::consts;
use signal_hook::iterator::Signals;

pub fn ignore(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigIgn) }
        .expect("sush(fatal): cannot ignore signal");
}

pub fn restore(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigDfl) }
        .expect("sush(fatal): cannot restore signal");
}

pub fn run_signal_check(core: &mut ShellCore) {
    for fd in 3..10 { //use FD 3~9 to prevent signal-hool from using these FDs
        nix::unistd::dup2(2, fd).expect("sush(fatal): init error");
    }

    core.sigint.store(true, Relaxed);
    let sigint = Arc::clone(&core.sigint);
 
    thread::spawn(move || {
        let mut signals = Signals::new(vec![consts::SIGINT])
                          .expect("sush(fatal): cannot prepare signal data");

        for fd in 3..10 { // release FD 3~9
            nix::unistd::close(fd).expect("sush(fatal): init error");
        }
        sigint.store(false, Relaxed);

        loop {
            thread::sleep(time::Duration::from_millis(100)); //0.1秒周期に変更
            for signal in signals.pending() {
                if signal == consts::SIGINT {
                    sigint.store(true, Relaxed);
                    eprintln!("^C");
                }
            }
        }
    });

    while core.sigint.load(Relaxed) {
        thread::sleep(time::Duration::from_millis(1));
    }

} //thanks: https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6

pub fn input_interrupt_check(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    if ! core.sigint.load(Relaxed) { //core.input_interrupt {
        return false;
    }

    core.sigint.store(false, Relaxed); //core.input_interrupt = false;
    core.db.exit_status = 130;
    feeder.consume(feeder.len());
    true
}

pub fn check_trap(core: &mut ShellCore) {
    let bkup = core.db.exit_status;

    let mut scripts = vec![];
    for t in &core.trapped {
        if t.0.load(Relaxed) {
            scripts.push(t.1.clone());
            t.0.store(false, Relaxed);
        }
    }

    for s in scripts {
        let mut feeder = Feeder::new(&s);
        let mut script = match Script::parse(&mut feeder, core, true) {
            Ok(None) => {continue;},
            Ok(s) => s.unwrap(),
            Err(e) => {e.print(core); continue;},
        };

        if let Err(e) = script.exec(core) {
            e.print(core);
        }
    }

    core.db.exit_status = bkup;
}
