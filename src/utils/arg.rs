//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub fn has_option(option: &str, args: &[String]) -> bool {
    args.iter().any(|arg| arg == option)
}

pub fn consume_arg(option: &str, args: &mut Vec<String>) -> bool {
    let found = args.iter().any(|arg| arg == option);
    if found {
        args.retain(|arg| arg != option);
    }
    found
}

pub fn consume_option(option: &str, args: &mut Vec<String>) -> bool {
    for (i, a) in args.iter().enumerate() {
        if a.starts_with("--") {
            return false;
        }

        if a == option {
            args.remove(i);
            return true;
        }
    }

    false
}

pub fn consume_starts_with(s: &str, args: &mut Vec<String>) -> Vec<String> {
    let mut ans = args.clone();
    ans.retain(|a| a.starts_with(s));
    args.retain(|a| !a.starts_with(s));
    ans
}

pub fn consume_with_next_arg(prev_opt: &str, args: &mut Vec<String>) -> Option<String> {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => match pos + 1 >= args.len() {
            true => None,
            false => {
                args.remove(pos);
                Some(args.remove(pos))
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
        }
        None => vec![],
    }
}

pub fn dissolve_option(opt: &str) -> Vec<String> {
    if opt.starts_with("--") {
        vec![opt.to_string()]
    } else if let Some(stripped) = opt.strip_prefix('-') {
        if stripped == "" {
            return vec!["-".to_string()];
        }

        stripped
            .chars()
            .map(|c| ("-".to_owned() + &c.to_string()).to_string())
            .collect()
    } else if let Some(stripped) = opt.strip_prefix('+') {
        if stripped == "" {
            return vec!["+".to_string()];
        }

        stripped
            .chars()
            .map(|c| ("+".to_owned() + &c.to_string()).to_string())
            .collect()
    } else {
        vec![opt.to_string()]
    }
}

pub fn dissolve_options(args: &[String]) -> Vec<String> {
    let mut ans = vec![];
    let mut stop = false;
    for a in args {
        if a == "--" {
            stop = true;
        }

        match stop {
            true => ans.push(a.to_string()),
            false => ans.append(&mut dissolve_option(a)),
        }
    }

    ans
}

pub fn dissolve_options_main() -> Vec<String> {
    let mut ans = vec![];
    let mut stop = false;
    for (i, a) in std::env::args().enumerate() {
        if i != 0 && !a.starts_with("-") || a == "--" {
            stop = true;
        }

        match stop {
            true => ans.push(a),
            false => ans.append(&mut dissolve_option(&a)),
        }
    }

    ans
}
