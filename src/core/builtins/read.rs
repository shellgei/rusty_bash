//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::{arg, error};

fn is_varname(s :&String) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();

    if '0' <= first_ch && first_ch <= '9' {
        return false;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
}

pub fn read_(core: &mut ShellCore, args: &mut Vec<String>, ignore_escape: bool) -> i32 {
    let mut remaining = String::new();
    let len = std::io::stdin()
        .read_line(&mut remaining)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    if len == 0 {
        return 1;
    }

    let ifs = match core.db.has_value("IFS") {
        true  => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    consume_ifs(&mut remaining, " \t");

    args.remove(0);
    while args.len() > 0 && ! remaining.is_empty() {
        let mut word = match eat_word(core, &mut remaining, &ifs, ignore_escape) {
            Some(w) => w,
            None => break,
        };

        if args.len() == 1 {
            let bkup = remaining.clone();
            consume_ifs(&mut remaining, &ifs);

            if remaining.is_empty() || remaining == "\n" {
            }else{
                word += &bkup;
            }
        }

        consume_tail_ifs(&mut word, " \t\n");

        if let Err(e) = core.db.set_param(&args[0], &word, None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }
        args.remove(0);
        consume_ifs(&mut remaining, &ifs);
    }

    0
}

pub fn read_a(core: &mut ShellCore, name: &String, ignore_escape: bool) -> i32 {
    let mut remaining = String::new();
    let len = std::io::stdin()
        .read_line(&mut remaining)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    if len == 0 {
        return 1;
    }

    let ifs = match core.db.has_value("IFS") {
        true  => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    consume_ifs(&mut remaining, " \t");

    let mut pos = 0;
    while ! remaining.is_empty() {
        let mut word = match eat_word(core, &mut remaining, &ifs, ignore_escape) {
            Some(w) => w,
            None => break,
        };
        consume_tail_ifs(&mut word, " \t\n");

        if let Err(e) = core.db.set_array_elem(name, &word, pos, None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }
        pos += 1;
        consume_ifs(&mut remaining, &ifs);
    }

    0
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let r_opt = arg::consume_option("-r", &mut args);
                                                      //
    if let Some(a) = arg::consume_with_next_arg("-a", &mut args) {
        return read_a(core, &a, r_opt);
    }

    for a in &args[1..] {
        if ! is_varname(&a) {
            eprintln!("bash: read: `{}': not a valid identifier", &a);
            return 1;
        }else{
            if let Err(e) = core.db.set_param(&a, "", None) {
                let msg = format!("{:?}", &e);
                error::print(&msg, core);
                return 1;
            }
        }
    }

    read_(core, &mut args, r_opt)
}

pub fn eat_word(core: &mut ShellCore, remaining: &mut String, ifs: &str, ignore_escape: bool) -> Option<String> {
    let mut esc = false;
    let mut pos = 0;
    let mut escape_pos = vec![];

    for c in remaining.chars() {
        if (esc || c == '\\') && ! ignore_escape {
            esc = ! esc;
            if esc {
                escape_pos.push(pos);
            }
            pos += c.len_utf8();
            continue;
        }

        if ifs.contains(c) {
            break;
        }
        pos += c.len_utf8();
    }

    if let Some(p) = escape_pos.last() {
        if p + 2 == remaining.len() && remaining.ends_with('\n') {
            remaining.pop();
            remaining.pop();

            let mut line = String::new();
            let len = std::io::stdin()
                .read_line(&mut line)
                .expect("SUSHI INTERNAL ERROR: Failed to read line");
        
            if len > 0 {
                *remaining += &line;
                return eat_word(core, remaining, ifs, ignore_escape);
                
            }
        }
    }

    let tail = remaining.split_off(pos);
    let mut ans = remaining.clone();
    *remaining = tail;

    for p in escape_pos {
        ans.remove(p);
    }


    Some(ans)
}

pub fn consume_tail_ifs(remaining: &mut String, ifs: &str) {
    loop {
        if let Some(c) = remaining.chars().last() {
            if ifs.contains(c) {
                remaining.pop();
                continue;
            }
        }
        break;
    }
}

pub fn consume_ifs(remaining: &mut String, ifs: &str) {
    let special_ifs: Vec<char> = ifs.chars().filter(|s| ! " \t\n".contains(*s)).collect(); 
    let mut pos = 0;
    let mut special_ifs_exist = false;

    for ch in remaining.chars() {
        if ! ifs.contains(ch) {
            break;
        }

        if special_ifs.contains(&ch) {
            if special_ifs_exist {
                break;
            }
            
            special_ifs_exist = true;
        }
        pos += ch.len_utf8();
    }

    let tail = remaining.split_off(pos);
    //dbg!("{:?}", &remaining);
    *remaining = tail;
}
