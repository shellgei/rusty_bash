//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io::{Write, BufRead, BufReader};
use std::fs::OpenOptions;
use std::collections::HashSet;

use crate::env;
use crate::ShellCore;
use crate::utils::{eval_glob, search_commands, chars_to_string};
use crate::term::Writer;
use crate::term::prompt;

fn passwd_to_home(line: String) -> Option<String> {
    let split = line.rsplit(':').collect::<Vec<&str>>();
    if let Some(s) = split.iter().nth(1) {
        return Some(s.to_string());
    }
    None
}

fn get_home(user: String) -> Option<String> {
    if let Ok(file) = OpenOptions::new().read(true).open("/etc/passwd"){
        let br = BufReader::new(file);
        for ln in br.lines() {
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
    }
    
    None
}

fn compare_nth_char(nth: usize, strs: &Vec<String>) -> bool {
    if strs.len() < 2 {
        return false;
    };

    let ch0: char;
    if let Some(ch) = &strs[0].chars().nth(nth){
        ch0 = *ch;
    }else{
        return false;
    }

    for s in strs {
        if let Some(ch) = s.chars().nth(nth){
            if ch != ch0{
                return false;
            }
        }else{
            return false;
        }
    }

    true
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

pub fn file_completion(writer: &mut Writer){
    let mut s: String = writer.last_arg() + "*";

    let tilde_user_pos = scanner_user_path(s.clone());
    let home = if tilde_user_pos == 1 {
        env::var("HOME").expect("Home is not set")
    }else if let Some(h) = get_home(s[1..tilde_user_pos].to_string()) {
        h
    }else{
        "".to_string()
    };

    if home.len() != 0 {
        s = s.replacen(&s[0..tilde_user_pos].to_string(), &home, 1);
    }

    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    //TODO: ~ should be replaced for other users.
    let home = env::var("HOME").expect("Home is not set");
    let base_len = writer.last_arg().len();
    if ans.len() == 1 {
        //let (x, y) = writer.cursor_pos();
        let a = if home.len() != 0 {
            ans[0].replacen(&home, &s[0..tilde_user_pos].to_string(), 1).clone()
        }else{
            ans[0].clone()
        };
        for ch in a[base_len..].chars() {
            writer.insert(ch);
        }
    }else{
        let a: Vec<String> = if home.len() != 0 {
            ans.iter().map(|x| x.replacen(&home, &s[0..tilde_user_pos].to_string(), 1)).collect()
        }else{
            ans
        };
        for (i, ch) in a[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &a) {
                writer.insert(ch);
            }else{
                break;
            }
        }
    }
}


pub fn show_file_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s: String = writer.last_arg() + "*";
    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    write!(writer.stdout, "\r\n").unwrap();
    for f in ans {
        write!(writer.stdout, "{}        ", f).unwrap();
    }
    write!(writer.stdout, "\r\n").unwrap();
    writer.stdout.flush().unwrap();
    prompt(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
    return;
}

pub fn command_completion(writer: &mut Writer){
    let paths = search_commands(&(writer.chars.iter().collect::<String>() + "*"));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    let base_len = writer.last_arg().len();
    if keys.len() == 1 {
        for ch in keys[0][base_len..].chars() {
            writer.insert(ch);
        }
        return;
    }else if keys.len() > 1 {
        for (i, ch) in keys[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &keys) {
                writer.insert(ch);
            }else{
                break;
            }
        }
        return;
    };
}

pub fn show_command_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let paths = search_commands(&(chars_to_string(&writer.chars) + "*"));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    write!(writer.stdout, "\r\n").unwrap();
    for f in keys {
        write!(writer.stdout, "{}        ", f).unwrap();
    }
    write!(writer.stdout, "\r\n").unwrap();
    writer.stdout.flush().unwrap();
    prompt(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
}

