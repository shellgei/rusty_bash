//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub fn has_option(option: &str, args: &[String]) -> bool {
    args.iter().any(|arg| arg == option)
}

fn add_prefix(prefix: char, opts: &str) -> Vec<String> {
    if opts.is_empty() {
        return vec![prefix.to_string()];
    }

    opts.chars().map(|c| format!("{prefix}{c}")).collect()
}

pub fn dissolve_option(arg: &str) -> Vec<String> {
    if arg.starts_with("--") {
        vec![arg.to_string()]
    } else if let Some(opts) = arg.strip_prefix('-') {
        add_prefix('-', opts)
    } else if let Some(opts) = arg.strip_prefix('+') {
        add_prefix('+', opts)
    } else {
        vec![arg.to_string()]
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
