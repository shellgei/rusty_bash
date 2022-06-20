//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use glob::glob;
use crate::env;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;

pub fn chars_to_string(chars: &Vec<char>) -> String {
    chars.iter().collect::<String>()
}

fn is_glob(s: &String) -> bool {
    //TODO: too crude
    for ch in s.chars() {
        if ch == '*' || ch == '[' || ch == '?' {
            return true;
        }
    }
    return false;
}

pub fn eval_glob(globstr: &String) -> Vec<String> {
    if ! is_glob(globstr) {
        return vec!();
    }

    let mut ans = vec!();
    let g = globstr.to_string();

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

pub fn combine_with(left: &Vec<String>, right: &Vec<String>, ch: &str) -> Vec<String> {
    if left.len() == 0 {
        return right.clone();
    };

    let mut ans = vec!();
    for lstr in left {
        let mut con = right
            .iter()
            .map(|r| lstr.clone() + ch + &r.clone())
            .collect();

        ans.append(&mut con);
    }
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

pub fn expand_tilde(path: &String) -> (String, String, String){
    let org_length = scanner_user_path(path.clone());
    let home = if org_length == 1 {
        env::var("HOME").expect("Home is not set")
    }else if org_length == 0{
        "".to_string()
    }else if let Some(h) = get_home(path[1..org_length].to_string()) {
        h
    }else{
        "".to_string()
    };

    let org = path[0..org_length].to_string();

    if home.len() != 0 {
        let h = home.clone();
        (path.replacen(&path[0..org_length].to_string(), &h, 1), home, org)
    }else{
        (path.to_string(), home, org)
    }
}

pub fn scanner_user_path(text: String) -> usize {
    if text.len() == 0 {
        return 0;
    }

    let mut pos = 0;
    for ch in text.chars() {
        if pos == 0 && ch != '~' {
            return 0;
        }

        if "/:\n *".find(ch) != None {
            break;
        }
        pos += ch.len_utf8();
    }

    pos
}


fn get_home(user: String) -> Option<String> {
    let file = if let Ok(f) = OpenOptions::new().read(true).open("/etc/passwd"){
        f
    }else{
        return None;
    };

    for ln in BufReader::new(file).lines() {
        if let Ok(line) = ln {
            if line.len() < user.len(){
                continue;
            }

            let split = line.split(':').collect::<Vec<&str>>();
            if let Some(u) = split.iter().nth(0){
                if u.to_string() == user {
                    return passwd_to_home(line);
                }
            }
        }
    }

    None
}

fn passwd_to_home(line: String) -> Option<String> {
    let split = line.rsplit(':').collect::<Vec<&str>>();
    if let Some(s) = split.iter().nth(1) {
        return Some(s.to_string());
    }
    None
}
