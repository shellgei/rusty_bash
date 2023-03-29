//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io::Write;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::ShellCore;
use crate::utils::{eval_glob, search_commands};
use crate::utils;
use crate::feeder::term::Writer;
use crate::feeder::term;

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
    if cands.len() == 0 {
        return String::new();
    }
    if cands.len() == 1 {
        return cands[0].clone();
    }

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


pub fn search_users(head: &String) -> Vec<String> {
    let mut ans = vec![];
    if let Ok(f) = File::open("/etc/passwd"){
        let reader = BufReader::new(f);
        for line in reader.lines(){
            let line = line.unwrap();
            if line.starts_with(head) {
                ans.push(line.split(":").nth(0).unwrap().to_string());
            }
        }
    }

    ans
}

fn user_completion_str(input: &String) -> String {
    if input.len() < 2 {
        return String::new();
    }
    if input.chars().nth(0).unwrap() != '~' {
        return String::new();
    }
    if let Some(_) = input.find("/") {
        return String::new();
    }

    let users = search_users(&input[1..].to_string());
    let completed_user = get_common_string(&users);

    if input.len() >= completed_user.len()+1 {
        return String::new();
    }

    completed_user[input.len()-1..].to_string()
}

fn get_completion_str(input: String) -> String {
    let user_completion = user_completion_str(&input);
    if user_completion.len() != 0 {
        return user_completion + "/";
    }

    let user = input.split("/").nth(0).unwrap().to_string();
    let s_org = input.replace("\\", "") + "*";
    let (s, home) = utils::tilde_to_dir(&s_org);

    let globed_paths = eval_glob(&s.replace("\\", ""));
    if globed_paths.len() == 0 || globed_paths[0].ends_with("*") {
        return "".to_string();
    };

    let tilde_paths = if let Some(home_path) = home {
        globed_paths.iter().map(|x| x.replacen(&home_path, &user, 1)).collect()
    }else{
        globed_paths
    };

    let mut base_len = input.replace("\\", "").len();
    if s.starts_with("./") {
        base_len -= 2;
    }

    let cands2 = tilde_paths.iter().map(|s| s[base_len..].to_string()).collect();
    return get_common_string(&cands2);
}

pub fn file_completion(writer: &mut Writer){
    let arg = writer.last_word();
    let chars = get_completion_str(arg);
    writer.insert_multi(chars.chars());
}


pub fn show_file_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let arg = writer.last_word();
    let user = arg.split("/").nth(0).unwrap().to_string();

    let ans = if arg.starts_with("~") && ! arg.contains("/") {//usr name completion
        search_users(&arg[1..].to_string())
            .iter()
            .map(|s| "~".to_owned() + s + "/")
            .collect()
    }else{//other completion
        let s = writer.last_word().replace("\\", "") + "*";
        let (rs, home) = utils::tilde_to_dir(&s);
        let globed_paths = eval_glob(&rs);
        let tilde_paths = if let Some(home_path) = home {
            globed_paths.iter().map(|x| x.replacen(&home_path, &user, 1)).collect()
        }else{
            globed_paths
        };
        tilde_paths
    };

    if ans.len() == 0 || ans[0].ends_with("*") {
        return;
    }

    write!(writer.stdout, "\r\n").unwrap();
    let ans2 = utils::align_elems_on_term(&ans, writer.terminal_size().0);
    write!(writer.stdout, "{}", ans2).unwrap();
    writer.stdout.flush().unwrap();
    term::prompt_normal(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
    return;
}

pub fn command_completion(writer: &mut Writer, core: &ShellCore){
    let s = writer.chars.iter().collect::<String>();

    let mut paths = search_commands(&(s.clone() + &"*"));
    paths.append(&mut utils::search_aliases(&s, core));
    paths.append(&mut utils::search_builtin(&s, core));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    let base_len = writer.last_word().len();
    let strings = keys.iter()
                      .map(|s| s[base_len..].to_string() + " ")
                      .collect();

    let common = get_common_string(&strings).replace("\\", "");
    writer.insert_multi(common.chars());
}

pub fn show_command_candidates(writer: &mut Writer, core: &mut ShellCore) {
    let s = writer.chars.iter().collect::<String>();

    let mut paths = search_commands(&(s.clone() + &"*"));
    paths.append(&mut utils::search_aliases(&s, core));
    paths.append(&mut utils::search_builtin(&s, core));

    let mut coms = HashSet::<String>::new();
    for p in paths {
        if let Some(com) = p.split("/").last() {
            coms.insert(com.to_string());
        };
    }

    let keys: Vec<String> = coms.into_iter().collect();

    write!(writer.stdout, "\r\n").unwrap();
    let ans2 = utils::align_elems_on_term(&keys, writer.terminal_size().0);
    write!(writer.stdout, "{}", ans2).unwrap();
    writer.stdout.flush().unwrap();
    term::prompt_normal(core);
    let (_, y) = writer.cursor_pos();
    writer.rewrite_line(y, writer.chars.iter().collect());
}

#[test]
fn file_candidates() {
    let comp_str = get_completion_str("/etc/passw".to_string());
    assert_eq!(comp_str, "d");

    let comp_str = get_completion_str("/li".to_string());
    assert_eq!(comp_str, "b");

    let comp_str = get_completion_str("~roo".to_string());
    assert_eq!(comp_str, "t/");
}

/*
#[test]
fn command_candidates() {
    let comp_str = command_completion_str("ech".to_string());
    assert_eq!(comp_str, "o");
}*/

#[test]
fn get_common_string_test() {
    let strs = vec![ "/aaa".to_string(), "/abb".to_string() ];
    assert_eq!(get_common_string(&strs), "/a");

    let strs = vec![ "./あいう".to_string(), "./あい".to_string(), "./あa".to_string() ];
    assert_eq!(get_common_string(&strs), "./あ");
}
