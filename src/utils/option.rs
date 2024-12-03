//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub fn consume_with_next_arg(prev_opt: &str, args: &mut Vec<String>) -> String {
    match args.iter().position(|a| a == prev_opt) {
        Some(pos) => {
            match pos+1 >= args.len() {
                true  => {
                    args.remove(pos);
                    "".to_string()
                },
                false => {
                    args.remove(pos);
                    args.remove(pos)
                },
            }
        },
        None => "".to_string(),
    }
}

fn dissolve_option(opt: &str) -> Vec<String> {
    if opt.starts_with("--") || ! opt.starts_with("-") {
        return vec![opt.to_string()];
    }

    opt[1..].chars().map(|c| ("-".to_owned() + &c.to_string()).to_string()).collect()
}

pub fn dissolve_options(args: &Vec<String>) -> Vec<String> {
    args.iter().map(|a| dissolve_option(a)).collect::<Vec<Vec<String>>>().concat()
}
