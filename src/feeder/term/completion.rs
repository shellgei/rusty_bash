//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io::Write;
use std::collections::HashSet;

use crate::ShellCore;
use crate::utils::{eval_glob, search_commands};
use crate::utils;
use crate::feeder::term::Writer;
use crate::feeder::term::prompt_normal;
use crate::utils::*;

fn compare_nth_char(nth: usize, strs: &Vec<String>) -> bool {
    if strs.len() == 0 {
        return false;
    };
    if strs.len() == 1 {
        return true;
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

fn get_common_string(cands: &Vec<String>) -> String {
    let mut chars = String::new();
    for (i, ch) in cands[0].chars().enumerate() {
        if compare_nth_char(i, &cands) {
            if ch == ' ' {
                chars += "\\";
            }
            chars += &ch.to_string();
        }else{
            break;
        }
    }
    return chars.to_string();
}

fn get_completion_str(input: String) -> String {
    let s: String = input.replace("\\", "") + "*";
    let org = s.clone();

    let candidates = eval_glob(&s.replace("\\", ""));
    if candidates.len() == 0 || candidates[0].ends_with("*") {
        return "".to_string();
    };

    let (s, home) = utils::tilde_to_dir(&s); //s: replaced path, home: home path
    let a: Vec<String> = if home.len() != 0 {
        candidates.iter().map(|x| x.replacen(&home, &org, 1)).collect()
    }else{
        candidates
    };

    let mut base_len = input.replace("\\", "").len();
    if s.starts_with("./") {
        base_len -= 2;
    }

    let cands2 = a.iter().map(|s| s[base_len..].to_string()).collect();
    return get_common_string(&cands2);
}

pub fn file_completion(writer: &mut Writer){
    let arg = writer.last_word();
    let chars = get_completion_str(arg);
    writer.insert_multi(chars.chars());
}


pub fn show_file_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s: String = writer.last_word().replace("\\", "") + "*";
    let (s, _) = utils::tilde_to_dir(&s); //s: replaced_path

    let ans = eval_glob(&s);
    if ans.len() == 0 || ans[0].ends_with("*") {
        return;
    }

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
    writer.stdout.flush().unwrap();
    prompt_normal(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
}

#[test]
fn file_candidates() {
    let comp_str = get_completion_str("/etc/passw".to_string());
    assert_eq!(comp_str, "d");

    let comp_str = get_completion_str("/li".to_string());
    assert_eq!(comp_str, "b");
}

#[test]
fn get_common_string_test() {
    let strs = vec![ "/aaa".to_string(), "/abb".to_string() ];
    assert_eq!(get_common_string(&strs), "/a");

    let strs = vec![ "./あいう".to_string(), "./あい".to_string(), "./あa".to_string() ];
    assert_eq!(get_common_string(&strs), "./あ");
}
