//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use nix::sys::signal;
use nix::sys::signal::{Signal, SigHandler};
use signal_hook::consts;
use signal_hook::iterator::Signals;
use std::{thread, time};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

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

pub fn input_interrupt_check(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    if ! core.sigint.load(Relaxed) { //core.input_interrupt {
        return false;
    }

    core.sigint.store(false, Relaxed); //core.input_interrupt = false;
    core.db.set_param("?", "130");
    feeder.consume(feeder.len());
    true
}
