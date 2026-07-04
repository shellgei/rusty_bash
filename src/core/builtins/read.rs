//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::error_;
use crate::elements::parameter::Parameter;
use crate::error::exec::ExecError;
use crate::{InputError, ShellCore, arg, error, utils};

fn read_(
    core: &mut ShellCore,
    args: &mut Vec<String>,
    ignore_escape: bool,
    limit: &mut usize,
    delim: &String,
    timeout: Option<f32>,
) -> i32 {
    let mut remaining =
        match utils::read_line_stdin_unbuffered(delim, timeout, core.is_subshell, limit) {
            Err(InputError::Timeout) => return 142,
            Ok(s) => s,
            Err(_) => "".to_string(),
        };

    if remaining.is_empty() {
        return 1;
    }

    let ifs = match core.db.exist("IFS") {
        true => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    let mut tail_space = ifs
        .chars()
        .filter(|i| " \t\n".contains(*i))
        .collect::<String>();
    tail_space += delim;

    args.remove(0);
    if args.is_empty() {
        args.push("REPLY".to_string());
        tail_space = "\n".to_string();
        if *limit < remaining.len() {
            let mut ifs_tmp = ifs.clone();
            ifs_tmp.retain(|e| " \t".contains(e));
            consume_ifs(&mut remaining, &ifs_tmp);
        }
    } else {
        let mut ifs_tmp = ifs.clone();
        ifs_tmp.retain(|e| " \t".contains(e));
        consume_ifs(&mut remaining, &ifs_tmp);
    }

    while !args.is_empty() && !remaining.is_empty()
    /* && *limit != 0*/
    {
        let (mut word, tail_escaped) = match eat_word(
            &mut remaining,
            &ifs,
            ignore_escape,
            delim,
            timeout,
            core,
            limit,
        ) {
            Err(_) => return 142,
            Ok(Some(w)) => w,
            Ok(None) => break,
        };

        //check_word_limit(&mut word, limit);

        if args.len() == 1
        /* && *limit != 0*/
        {
            let bkup = remaining.clone();
            consume_ifs(&mut remaining, &ifs);

            if remaining.is_empty() || remaining == "\n" {
            } else {
                word += &bkup;
            }
        }

        if !tail_escaped {
            consume_tail_ifs(&mut word, &tail_space, ignore_escape);
        }

        if let Err(e) = Parameter::parse_and_set(&args[0], &word, core) {
            return super::error_(1, "read", &String::from(&e), core);
        }

        args.remove(0);
        consume_ifs(&mut remaining, &ifs);
    }

    for a in args {
        if let Err(e) = Parameter::parse_and_set(a, "", core) {
            return super::error_(1, "read", &String::from(&e), core);
        }
    }

    0
}

fn read_a(
    core: &mut ShellCore,
    name: &str,
    ignore_escape: bool,
    limit: &mut usize,
    delim: &String,
    timeout: Option<f32>,
) -> i32 {
    let mut remaining =
        match utils::read_line_stdin_unbuffered(delim, timeout, core.is_subshell, limit) {
            Err(InputError::Timeout) => return 142,
            Ok(s) => s,
            Err(_) => "".to_string(),
        };

    if remaining.is_empty() {
        return 1;
    }

    let ifs = match core.db.exist("IFS") {
        true => core.db.get_param("IFS").unwrap(),
        false => " \t\n".to_string(),
    };

    let mut tail_space = ifs
        .chars()
        .filter(|i| " \t\n".contains(*i))
        .collect::<String>();
    tail_space += delim;

    consume_ifs(&mut remaining, " \t");

    let mut pos = 0;
    while !remaining.is_empty() {
        let (mut word, tail_escaped) = match eat_word(
            &mut remaining,
            &ifs,
            ignore_escape,
            delim,
            timeout,
            core,
            limit,
        ) {
            Err(_) => return 142,
            Ok(Some(w)) => w,
            Ok(None) => break,
        };
        //check_word_limit(&mut word, limit);
        if !tail_escaped {
            consume_tail_ifs(&mut word, &tail_space, ignore_escape);
        }

        if let Err(e) = core.db.set_array_elem(name, &word, pos, None, false) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }
        pos += 1;
        consume_ifs(&mut remaining, &ifs);
    }

    0
}

pub fn read(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.is_empty() {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let _e_opt = arg::consume_arg("-e", &mut args);
    let r_opt = arg::consume_arg("-r", &mut args);
    let mut timeout = match arg::consume_with_next_arg("-t", &mut args) {
        None => None,
        Some(s) => match s.parse::<f32>() {
            Ok(t) => {
                if t > 0.0 {
                    Some(t)
                } else if t < 0.0 {
                    return super::error(1, "read", &ExecError::InvalidTimeout(s), core);
                } else {
                    return 0;
                }
            }
            Err(_) => {
                return super::error(1, "read", &ExecError::InvalidTimeout(s), core);
            }
        },
    };

    if core.now_herestring {
        timeout = None;
    }

    let mut limit = usize::MAX;
    let mut delim = "\n".to_string();
    let mut limit_str = arg::consume_with_next_arg("-n", &mut args);
    if limit_str.is_none() {
        limit_str = arg::consume_with_next_arg("-N", &mut args);
        if limit_str.is_some() {
            delim = "".to_string();
        }
    }

    if let Some(c) = arg::consume_with_next_arg("-d", &mut args) {
        delim = c;
    }

    if let Some(limit_str) = limit_str {
        match limit_str.parse::<usize>() {
            Ok(n) => limit = n,
            Err(_) => {
                let err = format!("{}: invalid number", &limit_str);
                return error_(1, "read", &err, core);
            }
        };
    }

    let mut backup = None;
    let mut fdn = 0;
    if let Some(mut fd) = arg::consume_with_next_arg("-u", &mut args) {
        if fd.starts_with("-") {
            fd.remove(0);
        }

        if let Ok(n) = fd.parse::<i32>()
            && (3..256).contains(&n)
        {
            backup = Some(core.fds.backup(0));
            core.fds.read_used_fd = n;
            fdn = n;
            let _ = core.fds.replace(n, 0);
        }
    }

    let ans = if let Some(a) = arg::consume_with_next_arg("-a", &mut args) {
        if core.db.exist_nameref(&a) {
            let mut v = Parameter {
                text: a.clone(),
                name: a.clone(),
                ..Default::default()
            };
            if v.solve_nameref(core).is_err() {
                return super::error(1, "read", &ExecError::InvalidName(a.to_string()), core);
            }
            if v.index.is_some() {
                return super::error(1, "read", &ExecError::InvalidName(a.to_string()), core);
            }
        }

        let es = read_a(core, &a, r_opt, &mut limit, &delim, timeout);
        if es == 142 {
            let _ = core.db.unset(&a, None, core.shopts.query("localvar_unset"));
        }
        es
    } else {
        let es = read_(core, &mut args, r_opt, &mut limit, &delim, timeout);
        if es == 142 {
            for a in args {
                let _ = core.db.unset(&a, None, core.shopts.query("localvar_unset"));
            }
        }
        es
    };

    if let Some(fd) = backup {
        let _ = core.fds.replace(0, fdn);
        let _ = core.fds.replace(fd, 0);
    }

    ans
}

fn eat_word(
    remaining: &mut String,
    ifs: &str,
    ignore_escape: bool,
    delim: &String,
    timeout: Option<f32>,
    core: &mut ShellCore,
    limit: &mut usize,
) -> Result<Option<(String, bool)>, InputError> {
    //bool: tail space is escaped
    let mut esc = false;
    let mut pos = 0;
    let mut escape_pos = vec![];

    for c in remaining.chars() {
        if (esc || c == '\\') && !ignore_escape {
            esc = !esc;
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

    if let Some(p) = escape_pos.last()
        && p + 2 == remaining.len()
        && remaining.ends_with('\n')
    {
        remaining.pop();
        remaining.pop();

        let line = match utils::read_line_stdin_unbuffered(delim, timeout, core.is_subshell, limit)
        {
            Err(InputError::Timeout) => return Err(InputError::Timeout),
            Ok(s) => s,
            Err(_) => "".to_string(),
        };
        if !line.is_empty() {
            *remaining += &line;
            return eat_word(remaining, ifs, ignore_escape, delim, timeout, core, limit);
        }
    }

    let tail = remaining.split_off(pos);
    let mut ans = remaining.clone();
    *remaining = tail;
    let tail_escaped = tail_is_escaped(&ans) && ans.ends_with(" ");

    for p in escape_pos {
        ans.remove(p);
    }

    Ok(Some((ans, tail_escaped)))
}

fn tail_is_escaped(remaining: &str) -> bool {
    let mut esc = false;
    let mut ans = false;

    for c in remaining.chars() {
        if esc || c == '\\' {
            ans = esc;
            esc = !esc;
        } else {
            ans = false;
        }
    }

    ans
}

fn consume_tail_ifs(remaining: &mut String, ifs: &str, ignore_escape: bool) {
    let mut esc = false;
    if !ignore_escape {
        esc = tail_is_escaped(remaining);
    }

    if let Some(c) = remaining.chars().last()
        && ifs.contains(c)
    {
        remaining.pop();
        if esc {
            remaining.pop();
        }

        consume_tail_ifs(remaining, ifs, ignore_escape);
    }
}

fn consume_ifs(remaining: &mut String, ifs: &str) {
    let special_ifs: Vec<char> = ifs.chars().filter(|s| !" \t\n".contains(*s)).collect();
    let mut pos = 0;
    let mut special_ifs_exist = false;

    for ch in remaining.chars() {
        if !ifs.contains(ch) {
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
    *remaining = tail;
}
