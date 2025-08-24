//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn return_(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.source_function_level <= 0 {
        eprintln!("sush: return: can only `return' from a function or sourced script");
        return 2;
    }
    core.return_flag = true;

    if args.len() < 2 {
        return 0;
    } else if let Ok(n) = args[1].parse::<i32>() {
        return n % 256;
    }

    eprintln!("sush: return: {}: numeric argument required", args[1]);
    2
}

pub fn break_(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.loop_level <= 0 {
        eprintln!("sush: break: only meaningful in a `for', `while', or `until' loop");
        return 0;
    }

    core.break_counter += 1;
    if args.len() < 2 {
        return 0;
    }

    match args[1].parse::<i32>() {
        Ok(n) => {
            if n > 0 {
                core.break_counter += n - 1;
            } else {
                eprintln!("sush: break: {}: loop count out of range", args[1]);
                return 1;
            }
        }
        Err(_) => {
            eprintln!("sush: break: {}: numeric argument required", args[1]);
            return 128;
        }
    };
    0
}

pub fn continue_(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    if core.loop_level <= 0 {
        eprintln!("sush: continue: only meaningful in a `for', `while', or `until' loop");
        return 0;
    }

    core.continue_counter += 1;
    if args.len() < 2 {
        return 0;
    }

    match args[1].parse::<i32>() {
        Ok(n) => {
            if n > 0 {
                core.continue_counter += n - 1;
            } else {
                eprintln!("sush: continue: {}: loop count out of range", args[1]);
                return 1;
            }
        }
        Err(_) => {
            eprintln!("sush: continue: {}: numeric argument required", args[1]);
            return 128;
        }
    };
    0
}
