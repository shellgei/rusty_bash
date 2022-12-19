//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io::Write;
use std::collections::HashSet;

use crate::ShellCore;
use crate::utils::{eval_glob, search_commands, expand_tilde};
use crate::feeder::term::Writer;
use crate::feeder::term::prompt_normal;
use std::fs;
use crate::utils::*;

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

pub fn file_completion(writer: &mut Writer){
    let s: String = writer.last_word().replace("\\", "") + "*";
    let (s, home, org) = expand_tilde(&s);

    let ans = eval_glob(&s.replace("\\", ""));
    if ans.len() == 0 {
        return;
    };
    //eprintln!("\r\nANS: {:?}", ans);
    //ans = ans.iter().map(|a| a.replace(" ", "\\ ")).collect();

    let base_len = writer.last_word().len();
    let in_cur_dir = s.chars().nth(0) == Some('.') && s.chars().nth(1) == Some('/');

    if ans.len() == 1 {
        let add = if let Ok(_) = fs::read_dir(&ans[0]) {
            "/"
        }else{
            ""
        };

        let mut a = if home.len() != 0 {
            ans[0].replacen(&home, &org, 1).replace(" ", "\\ ")
        }else{
            ans[0].clone().replace(" ", "\\ ")
        } + add;

        if in_cur_dir {
            a = "./".to_owned() + &a;
        }

        writer.insert_multi(a[base_len..].chars());
    }else{
        let a: Vec<String> = if home.len() != 0 {
            ans.iter().map(|x| x.replacen(&home, &org, 1)).collect()
        }else{
            ans
        };
        //a = a.iter().map(|a| a.replace(" ", "\\ ")).collect();

        let mut chars = "".to_string();

        let mut base_len = writer.last_word().replace("\\", "").len();
        if in_cur_dir {
            base_len -= 2;
        }

        let ans2: Vec<String> = a.iter().map(|s| s[base_len..].to_string()).collect();

        for (i, ch) in ans2[0].chars().enumerate() {
            if compare_nth_char(i, &ans2) {
                if ch == ' ' {
                    chars += "\\";
                }
                chars += &ch.to_string();
            }else{
                break;
            }
        }
        writer.insert_multi(chars.chars());
    }
}


pub fn show_file_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s: String = writer.last_word().replace("\\", "") + "*";
    let (s, _, _) = expand_tilde(&s);

    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    write!(writer.stdout, "\r\n").unwrap();
    let ans2 = align_elems_on_term(&ans, writer.terminal_size().0);
    write!(writer.stdout, "{}", ans2).unwrap();
    writer.stdout.flush().unwrap();
    prompt_normal(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
    return;
}

pub fn command_completion(writer: &mut Writer, core: &ShellCore){
    let s = writer.chars.iter().collect::<String>();

    let mut paths = search_commands(&(s.clone() + &"*"));
    paths.append(&mut search_aliases(&s, core));
    paths.append(&mut search_builtin(&s, core));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    let base_len = writer.last_word().len();
    if keys.len() == 1 {
        writer.insert_multi(keys[0][base_len..].chars());
        return;
    }else if keys.len() > 1 {
        let mut ans = "".to_string();
        for (i, ch) in keys[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &keys) {
                ans += &ch.to_string();
            }else{
                break;
            }
        }
        writer.insert_multi(ans.chars());
        return;
    };
}

pub fn show_command_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s = writer.chars.iter().collect::<String>();

    let mut paths = search_commands(&(s.clone() + &"*"));
    paths.append(&mut search_aliases(&s, core));
    paths.append(&mut search_builtin(&s, core));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    write!(writer.stdout, "\r\n").unwrap();
    let ans2 = align_elems_on_term(&keys, writer.terminal_size().0);
    write!(writer.stdout, "{}", ans2).unwrap();
    /*
    write!(writer.stdout, "\r\n").unwrap();
    for f in keys {
        write!(writer.stdout, "{}        ", f).unwrap();
    }
    write!(writer.stdout, "\r\n").unwrap();
    */
    writer.stdout.flush().unwrap();
    prompt_normal(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
}

