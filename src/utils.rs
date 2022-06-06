//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use glob::glob;
use crate::env;

pub fn chars_to_string(chars: &Vec<char>) -> String {
    chars.iter().collect::<String>()
}

pub fn eval_glob(globstr: &String) -> Vec<String> {
    let mut ans = vec!();

    let mut g = globstr.clone();

    //expansion of tilde
    //TODO: ~root or ~other_user should be replaced but not implemented.
    let home = env::var("HOME").expect("Home is not set");
    let mut tilde_expansion = false;
    if g.len() > 0 {
        if let Some('~') = g.chars().nth(0) {
            tilde_expansion = true;
        }
        if g.len() > 1 {
            if let Some('*') = g.chars().nth(1) {
                tilde_expansion = false;
            }
        }
    }

    if tilde_expansion {
        g = g.replacen("~", &home, 1);
    };

    //TODO: too ugly
    if let Ok(path) = glob(&g) {
        for dir in path {
            if let Ok(d) = dir {
                if let Some(s) = d.to_str() {
                    if let Some('/') = g.chars().last() {
                        // the library omits the last / 
                        ans.push(s.to_string() + "/");
                    }else{
                        ans.push(s.to_string());
                    }
                };
            };
        };
    };
    ans
}

pub fn search_commands(globstr: &String) -> Vec<String> {
    let dirs = if let Ok(p) = env::var("PATH") {
        p.split(':').map(|s| s.to_string()).collect()
    }else{
        vec!()
    };

    let mut ans: Vec<String> = vec!();
    for d in dirs {
        if let Ok(path) = glob(&(d + "/" + globstr)) {
            for dir in path {
                if let Ok(d) = dir {
                    if let Some(s) = d.to_str() {
                        ans.push(s.to_string());
                    };
                };
            };
        };
    };

    ans
}

pub fn combine(left: &Vec<String>, right: &Vec<String>) -> Vec<String> {
    if left.len() == 0 {
        return right.clone();
    };

    let mut ans = vec!();
    for lstr in left {
        let mut con = right
            .iter()
            .map(|r| lstr.clone() + &r.clone())
            .collect();

        ans.append(&mut con);
    }
    ans
}

pub fn blue_string(strings: &Vec<String>) -> Vec<String> {
    strings
        .iter()
        .map(|s| format!("\x1b[34m{}\x1b[m", s))
        .collect()
}

