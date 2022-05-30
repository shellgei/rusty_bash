//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;
use std::io::{Write};

use crate::ShellCore;
use crate::utils::{eval_glob, search_commands, chars_to_string};

use crate::term::Writer;
use crate::term::prompt;


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
    let s: String = writer.last_arg() + "*";
    let ans = eval_glob(&s);
    if ans.len() == 0 {
        return;
    };

    let base_len = writer.last_arg().len();
    if ans.len() == 1 {
        for ch in ans[0][base_len..].chars() {
            writer.insert(ch);
        }
    }else{
        for (i, ch) in ans[0][base_len..].chars().enumerate() {
            if compare_nth_char(i+base_len, &ans) {
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

