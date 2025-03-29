//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::{arg, error};
use crate::builtins::error_exit;
use crate::elements::command;
use crate::elements::word::{Word, WordMode};

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

fn remove_escape(text: &str) -> String {
    let mut escape = false;
    let mut ans = String::new();
    for ch in text.chars() {
        if ch == '\\' {
            escape = !escape;
            if escape {
                continue;
            }
        }
        ans.push(ch);
    }
    ans
}

pub fn read_(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
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

    args.remove(0);
    while args.len() > 0 && ! remaining.is_empty() {
        consume_ifs(&mut remaining, &ifs);

        let mut ifs_ = ifs.clone();
        if args.len() == 1 {
            ifs_ = "".to_string();
        }

        let word = match eat_word(core, &mut remaining, &ifs_) {
            Some(w) => w,
            None => break,
        };

        if let Err(e) = core.db.set_param(&args[0], &word, None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }
        args.remove(0);
    }

    0
}

fn set_to_param(core: &mut ShellCore, args: &mut Vec<String>,
    pos: usize, word: &str) -> bool {
    if let Err(e) = core.db.set_param(&args[pos], word, None) {
        let msg = format!("{:?}", &e);
        error::print(&msg, core);
        return false;
    }
    true
}

pub fn read_a(core: &mut ShellCore, name: &String) -> i32 {
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");
    let mut feeder = Feeder::new(&line);
    let mut tmp = String::new();

    let mut pos = 0;
    loop {
        command::eat_blank_with_comment(&mut feeder, core, &mut tmp);
        if let Ok(Some(w)) = Word::parse(&mut feeder, core, Some(WordMode::ReadCommand)) {
            let text = remove_escape(&w.text);
            if let Err(_) = core.db.set_array_elem(name, &text, pos, None) {
                return error_exit(1, "read", "array allocation error", core);
            }
            pos +=1;
        }else{
            break;
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
}

pub fn read_r(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut line = String::new();
    let len = std::io::stdin()
        .read_line(&mut line)
        .expect("SUSHI INTERNAL ERROR: Failed to read line");

    let mut pos = 1;
    let mut surplus = vec![];
    for w in line.trim_end().split(' ') {
        if pos < args.len()-1 {
            if ! set_to_param(core, args, pos, &w) {
                return 1;
            }
            pos +=1;
        }else{
            surplus.push(w);
        }
    }

    if ! surplus.is_empty() {
        if ! set_to_param(core, args, args.len()-1, &surplus.join(" ")) {
            return 1;
        }
    }

    match len == 0 {
        true  => 1,
        false => 0,
    }
}

pub fn read(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    if let Some(a) = arg::consume_with_next_arg("-a", &mut args) {
        return read_a(core, &a);
    }
    let r_opt = arg::consume_option("-r", &mut args); //TODO: change the precedure

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

    match r_opt {
        true  => read_r(core, &mut args),
        false => read_(core, &mut args),
    }
}

pub fn eat_word(core: &mut ShellCore, remaining: &mut String, ifs: &str) -> Option<String> {
    let mut esc = false;
    let mut pos = 0;
    let mut escape_pos = vec![];

    for c in remaining.chars() {
        pos += c.len_utf8();
        if esc || c == '\\' {
            esc = ! esc;
            if esc {
                escape_pos.push(pos-1);
            }
            continue;
        }

        if ifs.contains(c) {
            break;
        }
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
                return eat_word(core, remaining, ifs);
                
            }
        }
    }

    let tail = remaining.split_off(pos);
    let mut ans = remaining.clone();
    *remaining = tail;

    for p in escape_pos {
        ans.remove(p);
    }

    if ans.ends_with("\n") {
        ans.pop();
    }

    Some(ans)
}

pub fn consume_ifs(remaining: &mut String, ifs: &str) {
    let mut pos = 0;
    let mut esc = false;

    for ch in remaining.chars() {
        if esc || ch == '\\' {
            esc = ! esc;
            pos += ch.len_utf8();
            continue;
        }

        if ! ifs.contains(ch) {
            break;
        }
        pos += ch.len_utf8();
    }

    let tail = remaining.split_off(pos);
    *remaining = tail;
}
