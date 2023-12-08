//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;
mod elements;

use std::{env, process, thread, time};
use crate::core::ShellCore;
use crate::elements::script::Script;
use crate::feeder::Feeder;
use signal_hook::consts;
use signal_hook::iterator::Signals;
use std::sync::{Arc, Mutex};

fn show_version() {
    eprintln!("Sushi Shell 202305_5");
    eprintln!("© 2023 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
}

fn run_signal_check(core: &mut ShellCore) {
    let mut arc = Arc::clone(&core.signal_flags); //追加
 
    thread::spawn(move || { //クロージャの処理全体を{}で囲みましょう
        let mut signals = Signals::new(vec![consts::SIGINT])
                          .expect("sush(fatal): cannot prepare signal data");

        loop {
            thread::sleep(time::Duration::from_millis(100)); //0.1秒周期に変更
            for signal in signals.pending() {
                check_signals(signal, &mut arc);
            }
        }
    });
} //thanks: https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6

fn check_signals(signal: i32, arc: &mut Arc<Mutex<Vec<bool>>>) {
    match signal {
        consts::SIGINT => {
            let mut flags = arc.lock().unwrap();
            flags[consts::SIGINT as usize] = true;
            eprintln!("\nCOME HERE\n"); //確認用
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
    if ! core.check_signal(consts::SIGINT) { //core.input_interrupt {
        return false;
    }

    core.unset_signal(consts::SIGINT); //core.input_interrupt = false;
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
                    s.exec(core)
                }
            },
            None => continue,
        }
    }
    core.exit();
}
