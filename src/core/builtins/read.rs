//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::{arg, error, signal};
use crate::InputError;
use crate::elements::command;
use crate::elements::word::Word;

fn is_varname(s :&String) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();

    if '0' <= first_ch && first_ch <= '9' {
        return false;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
}

pub fn read_(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut feeder = Feeder::new("");
    let mut tmp = String::new();

    loop {
        if let Err(e) = core.jobtable_check_status() {
            e.print(core);
        }
        core.jobtable_print_status_change();

        match feeder.feed_line(core) {
            Ok(()) => {}, 
            Err(InputError::Interrupt) => {
                signal::input_interrupt_check(&mut feeder, core);
                signal::check_trap(core);
                continue;
            },
            _ => break,
        }
        dbg!("HERE");
        command::eat_blank_with_comment(&mut feeder, core, &mut tmp);

        if let Ok(Some(w)) = Word::parse(&mut feeder, core, false) {
            dbg!("{:?}", &w.text);
            continue;
        }

        break;
    }

    /*
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    let mut pos = 1;
    let mut overflow = String::new();
    for w in line.trim_end().split(' ') {
        if pos < args.len()-1 {
            if let Err(e) = core.db.set_param(&args[pos], &w, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
            pos += 1;
        }else{
            if overflow.len() != 0 {
                overflow += " ";
            }
            overflow += &w;
            if let Err(e) = core.db.set_param(&args[pos], &overflow, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
    */
    0
}

pub fn read_r(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    let mut pos = 1;
    let mut overflow = String::new();
    for w in line.trim_end().split(' ') {
        if pos < args.len()-1 {
            if let Err(e) = core.db.set_param(&args[pos], &w, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
            pos += 1;
        }else{
            if overflow.len() != 0 {
                overflow += " ";
            }
            overflow += &w;
            if let Err(e) = core.db.set_param(&args[pos], &overflow, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let r_opt = arg::consume_option("-r", &mut args); //TODO: change the precedure

    for a in &args[1..] {
        if ! is_varname(&a) {
            eprintln!("bash: read: `{}': not a valid identifier", &a);
            return 1;
        }else{
            if let Err(e) = core.db.set_param(&a, "", None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }


    match r_opt {
        true  => read_r(core, &mut args),
        false => read_(core, &mut args),
    }
    /*
    //TODO: this procedure may be for -r option
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    let mut pos = 1;
    let mut overflow = String::new();
    for w in line.trim_end().split(' ') {
        if pos < args.len()-1 {
            if let Err(e) = core.db.set_param(&args[pos], &w, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
            pos += 1;
        }else{
            if overflow.len() != 0 {
                overflow += " ";
            }
            overflow += &w;
            if let Err(e) = core.db.set_param(&args[pos], &overflow, None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
    */
}
