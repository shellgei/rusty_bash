//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::signal;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::{thread, time};
use signal_hook::iterator::Signals;

pub fn trap(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut signals = vec![];
    let mut signals_i32 = vec![];
    for a in &args[2..] {
        if let Ok(n) = a.parse::<i32>() {
            signals.push(TryFrom::try_from(n).unwrap());
            signals_i32.push(n);
        }else{
            return 1;
        }
    }

    for s in &signals {
        signal::ignore(*s);
    }

    core.trapped.push(Arc::new(AtomicBool::new(false)));

    let trap = Arc::clone(&core.trapped.last().unwrap());

    thread::spawn(move || {
        let mut signals = Signals::new(signals_i32.clone())
                          .expect("sush(fatal): cannot prepare signal data");

        loop {
            thread::sleep(time::Duration::from_millis(100)); //0.1秒周期に変更
            for signal in signals.pending() {
                if signals_i32.contains(&signal) {
                    trap.store(true, Relaxed);
                }
            }
        }
    });

    0
}
