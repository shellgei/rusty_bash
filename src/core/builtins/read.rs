//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, error, ShellCore, utils};
use crate::elements::substitution::variable::Variable;
use super::error_exit;

fn check_word_limit(word: &mut String, limit: &mut usize) -> bool {
    let mut pos = 0;
    for c in word.chars() {
        if *limit == 0 {
            let _ = word.split_off(pos);
            return true;
        }
        *limit -= 1;

        pos += c.len_utf8();
    }
    false
}

pub fn read_(core: &mut ShellCore, args: &mut Vec<String>,
             ignore_escape: bool, limit: &mut usize, delim: &String) -> i32 {
    let mut remaining = utils::read_line_stdin_unbuffered(delim)
                        .unwrap_or("".to_string());
    if remaining.is_empty() {
        return 1;
    }

    let ifs = match core.db.has_value("IFS") {
        true  => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    let mut tail_space = ifs.chars().filter(|i| " \t\n".contains(*i)).collect::<String>();
    tail_space += delim;

    args.remove(0);
    if args.len() == 0 {
        args.push("REPLY".to_string());
    }

    consume_ifs(&mut remaining, " \t", limit);

    while args.len() > 0 && ! remaining.is_empty() && *limit != 0 {
        let mut word = match eat_word(core, &mut remaining, &ifs, ignore_escape, delim) {
            Some(w) => w,
            None => break,
        };

        check_word_limit(&mut word, limit);

        if args.len() == 1 && *limit != 0 {
            let bkup = remaining.clone();
            consume_ifs(&mut remaining, &ifs, limit);

            if remaining.is_empty() || remaining == "\n" {
            }else{
                word += &bkup;
            }
        }

        consume_tail_ifs(&mut word, &tail_space);
        
        if let Err(e) = Variable::parse_and_set(&args[0], &word, core) {
            e.print(core);
            return 1;
        }

        args.remove(0);
        consume_ifs(&mut remaining, &ifs, limit);
    }

    0
}

pub fn read_a(core: &mut ShellCore, name: &String, ignore_escape: bool,
              limit: &mut usize, delim: &String) -> i32 {
    let mut remaining = utils::read_line_stdin_unbuffered(delim)
                        .unwrap_or("".to_string());
    if remaining.is_empty() {
        return 1;
    }

    let ifs = match core.db.has_value("IFS") {
        true  => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    let mut tail_space = ifs.chars().filter(|i| " \t\n".contains(*i)).collect::<String>();
    tail_space += delim;

    consume_ifs(&mut remaining, " \t", limit);

    let mut pos = 0;
    while ! remaining.is_empty() {
        let mut word = match eat_word(core, &mut remaining, &ifs, ignore_escape, delim) {
            Some(w) => w,
            None => break,
        };
        check_word_limit(&mut word, limit);
        consume_tail_ifs(&mut word, &tail_space);

        if let Err(e) = core.db.set_array_elem(name, &word, pos, None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }
        pos += 1;
        consume_ifs(&mut remaining, &ifs, limit);
    }

    0
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let r_opt = arg::consume_option("-r", &mut args);
    let mut limit = std::usize::MAX;
    let limit_str = arg::consume_with_next_arg("-n", &mut args);
    let delim = match arg::consume_with_next_arg("-d", &mut args) {
        Some(c) => c,
        None    => "\n".to_string(),
    };

    if limit_str.is_some() {
        let s = limit_str.unwrap();
        match s.parse::<usize>() {
            Ok(n) => limit = n,
            Err(_) => {
                let err = format!("{}: invalid number", &s);
                return error_exit(1, "read", &err, core);
            },
        };
    }

    if let Some(a) = arg::consume_with_next_arg("-a", &mut args) {
        return read_a(core, &a, r_opt, &mut limit, &delim);
    }

    read_(core, &mut args, r_opt, &mut limit, &delim)
}

pub fn eat_word(core: &mut ShellCore, remaining: &mut String,
                ifs: &str, ignore_escape: bool, delim: &String) -> Option<String> {
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

            let line = utils::read_line_stdin_unbuffered(delim).unwrap_or("".to_string());
            if line.len() > 0 {
                *remaining += &line;
                return eat_word(core, remaining, ifs, ignore_escape, delim);
                
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

pub fn consume_ifs(remaining: &mut String, ifs: &str, limit: &mut usize) {
    let special_ifs: Vec<char> = ifs.chars().filter(|s| ! " \t\n".contains(*s)).collect(); 
    let mut pos = 0;
    let mut special_ifs_exist = false;

    for ch in remaining.chars() {
        if ! ifs.contains(ch) || *limit == 0 {
            break;
        }

        if special_ifs.contains(&ch) {
            if special_ifs_exist {
                break;
            }
            
            special_ifs_exist = true;
        }
        pos += ch.len_utf8();
        *limit -= 1;
    }

    let tail = remaining.split_off(pos);
    *remaining = tail;
}
