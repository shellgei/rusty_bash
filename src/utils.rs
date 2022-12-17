//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use glob::glob;
use crate::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use crate::ShellCore;

pub fn chars_to_string(chars: &Vec<char>) -> String {
    chars.iter().collect::<String>()
}

fn is_glob(s: &String) -> bool {
    let mut escaped = false;

    for ch in s.chars() {
        if escaped {
            continue;
        }else if ! escaped && ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '*' || ch == '[' || ch == '?' {
            return true;
        }
    }
    return false;
}

pub fn eval_glob(globstr: &String) -> Vec<String> {
    if ! is_glob(&globstr) {
        return vec!(globstr.clone());
    }

    let mut ans = vec![];
    let g = globstr.replace("\\ ", " ").to_string();

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

    if ans.len() == 0 {
        return vec!(globstr.clone());
    }

    ans
}

pub fn search_commands(globstr: &String) -> Vec<String> {
    let dirs = if let Ok(p) = env::var("PATH") {
        p.split(':').map(|s| s.to_string()).collect()
    }else{
        vec![]
    };

    let mut ans: Vec<String> = vec![];
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

pub fn search_builtin(head: &String, core: &ShellCore) -> Vec<String> {
    let len = head.len();

    let mut ans = vec![];
    for a in core.builtins.keys() {
        if a.len() >= len && &a[0..len] == head {
            ans.push(a.clone());
        }
    }

    ans
}

pub fn search_aliases(head: &String, core: &ShellCore) -> Vec<String> {
    let len = head.len();

    let mut ans = vec![];
    for a in core.aliases.keys() {
        if a.len() >= len && &a[0..len] == head {
            ans.push(a.clone());
        }
    }

    ans
}

pub fn combine_with(left: &Vec<String>, right: &Vec<String>, ch: &str) -> Vec<String> {
    if left.len() == 0 {
        return right.clone();
    };

    let mut ans = vec![];
    for lstr in left {
        let mut con = right
            .iter()
            .map(|r| lstr.clone() + ch + &r.clone())
            .collect();

        ans.append(&mut con);
    }
    ans
}

pub fn combine(left: &mut Vec<Vec<String>>, right: Vec<Vec<String>>) -> Vec<Vec<String>> {
    if left.len() == 0 {
        return right;
    };

    let mut ans = vec![];
    for lv in left {
        let lv_len = lv.len();
        for rv in &right {
            let mut clv = lv.clone();
            clv.append(&mut rv.clone());
            let n = clv[lv_len].clone();
            clv[lv_len-1] += &n;
            clv.remove(lv_len);
            ans.push(clv);
        }
    }
    ans
}

pub fn blue_strings(strings: &Vec<String>) -> Vec<String> {
    strings
        .iter()
        .map(|s| format!("\x1b[34m{}\x1b[m", s))
        .collect()
}

pub fn blue_string(s: &String) -> String {
    format!("\x1b[34m{}\x1b[m", s)
}

pub fn expand_tilde(path: &String) -> (String, String, String){
    let org_length = path.len();
    let home = if org_length == 1 {
        env::var("HOME").expect("Home is not set")
    }else if org_length == 0{
        "".to_string()
    }else if let Some(h) = get_home(path[1..].to_string()) {
        h
    }else{
        path.to_string()//"".to_string()
    };

    let org = path[0..org_length].to_string();

    if home.len() != 0 {
        let h = home.clone();
        (path.replacen(&path[0..org_length].to_string(), &h, 1), home, org)
    }else{
        (path.to_string(), home, org)
    }
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

fn get_column_num(list: &Vec<String>, width: u32) -> (usize, Vec<u32>) {
    let lens: Vec<usize> = list.iter().map(|s| s.len()).collect();

    let mut ans = 1;
    let mut colwids_ans = vec![];
    for colnum in 1..100 {
        let mut line_num = list.len() / colnum;
        if list.len() % colnum != 0 {
            line_num += 1;
        };

        let mut n = 0;
        let mut wid = 0;
        let mut colwids = vec![];
        while n < list.len() {
            let colwid = if n+line_num < lens.len() {
                *(lens[n..n+line_num].iter().max().unwrap()) as u32
            }else{
                *(lens[n..].iter().max().unwrap()) as u32
            } + 2;

            colwids.push(colwid);
            wid += colwid;
            n += line_num;
        }

        if wid < width {
            ans = colnum;
            colwids_ans = colwids;
        }else{
            break;
        }
    }

    (ans, colwids_ans)
}

pub fn align_elems_on_term(list: &Vec<String>, width: u32) -> String {
    let (colnum, colwids) = get_column_num(&list, width);

    let mut line_num = list.len() / colnum;
    if list.len() % colnum != 0 {
        line_num += 1;
    };

    let mut ans = "".to_string();
    for row in 0..line_num {
        for col in 0..colnum {
            let pos = col*line_num + row;
            if pos < list.len() {
                ans += &(list[pos].clone());
                for _ in 0..(colwids[col] - list[pos].len() as u32) {
                    ans += " ";
                }
            }
        }
        ans += "\r\n";
    }
    ans
}


pub fn get_fullpath(com: &String) -> String {
    let dirs = if let Ok(p) = env::var("PATH") {
        p.split(':').map(|s| s.to_string()).collect()
    }else{
        vec![]
    };

    for d in dirs {
        let path = d + "/" + com;
        if fs::metadata(&path).is_ok() {
            return path;
        }
    }

    "".to_string()
}
