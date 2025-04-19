//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub fn consume_option(opt: &str, args: &mut Vec<String>) -> bool {
    match args.iter().position(|a| a == opt) {
        Some(pos) => {
            args.remove(pos);
            true
        },
        None => false,
    }
}

pub fn consume_with_next_arg(prev_opt: &str, args: &mut Vec<String>) -> Option<String> {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            match pos+1 >= args.len() {
                true  => None,
                false => {
                    args.remove(pos);
                    Some(args.remove(pos))
                },
            }
        },
        None => None,
    }
}

pub fn consume_with_subsequents(prev_opt: &str, args: &mut Vec<String>) -> Vec<String> {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            let ans = args[pos..].to_vec();
            *args = args[..pos].to_vec();
            ans
        },
        None => vec![],
    }
}

/*
pub fn replace_to_short_opt(opt1: &str, opt2: &str, to: &str, args: &mut Vec<String>) {
    if let Some(pos) = args.iter().position(|a| a == opt1) {
        if args.len() > pos+1 && args[pos+1] == opt2 {
            args.remove(pos);
            args.remove(pos);
            args.insert(pos, to.to_string());
        }
    }
}
*/

fn dissolve_option(opt: &str) -> Vec<String> {
    if opt.starts_with("-") {
        opt[1..].chars().map(|c| ("-".to_owned() + &c.to_string()).to_string()).collect()
    }else if opt.starts_with("+") {
        opt[1..].chars().map(|c| ("+".to_owned() + &c.to_string()).to_string()).collect()
    }else {
        vec![opt.to_string()]
    }
    /*
    if opt.starts_with("--") || ! opt.starts_with("-") {
        return vec![opt.to_string()];
    }

    opt[1..].chars().map(|c| ("-".to_owned() + &c.to_string()).to_string()).collect()
    */
}

pub fn dissolve_options(args: &Vec<String>) -> Vec<String> {
    args.iter().map(|a| dissolve_option(a)).collect::<Vec<Vec<String>>>().concat()
}

/*
pub fn consume_after_options(args: &mut Vec<String>, start: usize) -> Vec<String> {
    let mut has_option = false;

    for (i, arg) in args[start..].iter().enumerate() {
        if arg == "--" {
            args.remove(i+start);
            return args.split_off(i+start);
        }

        if arg.starts_with("-") {
            has_option = true;
            continue;
        }

        if has_option {
            return args.split_off(i+start);
        }
    }

    args.split_off(start)
}
*/
