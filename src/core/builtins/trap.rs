//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::signal;
use crate::ShellCore;
use nix::sys::signal::Signal;
use signal_hook::iterator::Signals;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::{thread, time};

pub fn trap(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() == 1 {
        for e in &core.traplist {
            if e.0 == 0 {
                println!("trap -- '{}' EXIT", &e.1);
            } else if let Ok(s) = Signal::try_from(e.0) {
                println!("trap -- '{}' {}", &e.1, &s);
            }
        }
        return 0;
    }

    if args.len() < 3 {
        // TODO: print the list of trap entries if args.len() == 1
        eprintln!("trap: usage: trap arg signal_spec ...");
        return 2;
    }

    let forbiddens = Vec::from(signal_hook::consts::FORBIDDEN);
    let signals = match args_to_nums(&args[2..], &forbiddens) {
        Ok(v) => v,
        Err(e) => {
            e.print(core);
            return 1;
        }
    };

    let mut exit = false;
    let mut valid_signals = vec![];
    for n in &signals {
        if *n == 0 {
            exit = true;
            continue;
        }

        if let Ok(s) = TryFrom::try_from(*n) {
            signal::ignore(s);
            valid_signals.push(*n);
            continue;
        };

        let msg = format!("trap: {n}: invalid signal specification");
        return super::error_exit(1, &args[0], &msg, core);
    }

    if !valid_signals.is_empty() {
        for n in &valid_signals {
            core.traplist.push((*n, args[1].to_string()));
        }
        run_thread(valid_signals, &args[1], core);
    }

    if exit {
        core.traplist.push((0, args[1].to_string()));
        core.exit_script = args[1].clone();
    }

    0
}

fn run_thread(signal_nums: Vec<i32>, script: &str, core: &mut ShellCore) {
    core.trapped
        .push((Arc::new(AtomicBool::new(false)), script.to_string()));

    let trap = Arc::clone(&core.trapped.last().unwrap().0);

    thread::spawn(move || {
        let mut signals =
            Signals::new(signal_nums.clone()).expect("sush(fatal): cannot prepare signal data");

        loop {
            thread::sleep(time::Duration::from_millis(5));
            for signal in signals.pending() {
                if signal_nums.contains(&signal) {
                    trap.store(true, Relaxed);
                }
            }
        }
    });
}

fn arg_to_num(arg: &str, forbiddens: &[i32]) -> Result<i32, ExecError> {
    if arg == "EXIT" || arg == "0" {
        return Ok(0);
    }

    if let Ok(n) = Signal::from_str(arg) {
        return Ok(n as i32);
    }

    if let Ok(n) = Signal::from_str(&("SIG".to_owned() + arg)) {
        return Ok(n as i32);
    }

    if let Ok(n) = arg.parse::<i32>() {
        if forbiddens.contains(&n) {
            return Err(ExecError::Other(format!(
                "trap: {arg}: forbidden signal for trap"
            )));
        }
        return Ok(n);
    }

    Err(ExecError::Other(format!(
        "trap: {arg}: invalid signal specification"
    )))
}

fn args_to_nums(args: &[String], forbiddens: &[i32]) -> Result<Vec<i32>, ExecError> {
    let mut ans = vec![];
    for a in args {
        let n = arg_to_num(a, forbiddens)?;
        ans.push(n);
    }
    Ok(ans)
}
