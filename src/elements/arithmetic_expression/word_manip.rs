//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::Elem;
use super::syntax_error_msg;
use crate::elements::arithmetic_expression::Word;

pub fn to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<Elem, String> {
    if pre_increment != 0 && post_increment != 0 
    || w.text.find('\'').is_some() {
        return Err(syntax_error_msg(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    let res = match pre_increment {
        0 => change_variable(&name, core, post_increment, false),
        _ => change_variable(&name, core, pre_increment, true),
    };

    match res {
        Ok(n)  => return Ok(Elem::Operand(n)),
        Err(e) => return Err(e),
    }
}

fn to_num(w: &Word, core: &mut ShellCore) -> Result<i64, String> {
    if w.text.find('\'').is_some() {
        return Err(syntax_error_msg(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    str_to_num(&name, core)
}

pub fn substitute(op: &str, w: &Word, right_value: i64, core: &mut ShellCore)
                                      -> Result<Elem, String> {
    if w.text.find('\'').is_some() {
        return Err(syntax_error_msg(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    match op {
        "=" => {
            core.data.set_param(&name, &right_value.to_string());
            return Ok(Elem::Operand(right_value));
        },
        _   => {},
    }

    let current_num = match to_num(w, core) {
        Ok(n)  => n,
        Err(e) => return Err(e),
    };

    let new_value = match op {
        "+=" => current_num + right_value,
        "-=" => current_num - right_value,
        "*=" => current_num * right_value,
        "&="  => current_num & right_value,
        "^="  => current_num ^ right_value,
        "|="  => current_num | right_value,
        "<<="  => if right_value < 0 {0} else {current_num << right_value},
        ">>="  => if right_value < 0 {0} else {current_num >> right_value},
        "/=" | "%=" => {
            if right_value == 0 {
                return Err("divided by 0".to_string());
            }
            if op == "%=" {
                current_num % right_value
            }else{
                current_num / right_value
            }
        },
        _   => 0,
    };

    core.data.set_param(&name, &new_value.to_string());
    Ok(Elem::Operand(new_value))
}


fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

fn recursion_error(token: &str) -> String {
    format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token)
}

fn str_to_num(name: &str, core: &mut ShellCore) -> Result<i64, String> {
    let mut name = name.to_string();

    const RESOLVE_LIMIT: i32 = 10000;

    for i in 0..RESOLVE_LIMIT {
        match is_name(&name, core) {
            true  => name = core.data.get_param(&name),
            false => break,
        }

        if i == RESOLVE_LIMIT - 1 {
            return Err(recursion_error(&name));
        }
    }

    if let Some(n) = parse_as_i64(&name) {
        Ok( n )
    }else if is_name(&name, core) {
        Ok( 0 )
    }else{
        Err(syntax_error_msg(&name))
    }
}

fn change_variable(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<i64, String> {
    if ! is_name(name, core) {
        return match inc != 0 && ! pre {
            true  => Err(syntax_error_msg(name)),
            false => str_to_num(&name, core),
        }
    }

    let num_i64 = match str_to_num(&name, core) {
        Ok(n)        => n,
        Err(err_msg) => return Err(err_msg), 
    };
    
    core.data.set_param(name, &(num_i64 + inc).to_string());

    match pre {
        true  => Ok(num_i64+inc),
        false => Ok(num_i64),
    }
}

fn parse_with_base(base: i64, s: &mut String) -> Option<i64> {
    //dbg!("{:?} {:?}", &base, &s);
    let mut ans = 0;
    for ch in s.chars() {
        ans *= base;
        let num = if ch >= '0' && ch <= '9' {
            ch as i64 - '0' as i64
        }else if ch >= 'a' && ch <= 'z' {
            ch as i64 - 'a' as i64 + 10
        }else if ch >= 'A' && ch <= 'Z' {
            if base <= 36 {
                ch as i64 - 'A' as i64 + 10
            }else{
                ch as i64 - 'A' as i64 + 36
            }
        }else if ch == '@' {
            62
        }else if ch == '_' {
            63
        }else{
            return None;
        };

        match num < base {
            true  => ans += num,
            false => return None,
        }
    }

    Some(ans)
}

fn get_sign(s: &mut String) -> String {
    *s = s.trim().to_string();
    match s.starts_with("+") || s.starts_with("-") {
        true  => {
            let c = s.remove(0).to_string();
            *s = s.trim().to_string();
            c
        },
        false => "+".to_string(),
    }
}

fn get_base(s: &mut String) -> Option<i64> {
    if s.starts_with("0x") || s.starts_with("0X") {
        s.remove(0);
        s.remove(0);
        return Some(16);
    }

    if s.starts_with("0") {
        s.remove(0);
        return Some(8);
    }

    if let Some(n) = s.find("#") {
        let base_str = s[..n].to_string();
        *s = s[(n+1)..].to_string();
        return match base_str.parse::<i64>() {
            Ok(n) => Some(n),
            _     => None,
        };
    }

    Some(10)
}

pub fn parse_as_i64(s: &str) -> Option<i64> {
    if s.find('\'').is_some() {
        return None;
    }
    if s.len() == 0 {
        return Some(0);
    }

    let mut sw = s.to_string();
    let sign = get_sign(&mut sw);
    let base = match get_base(&mut sw) {
        Some(n) => n, 
        _       => return None,
    };

    match ( parse_with_base(base, &mut sw), sign.as_str() ) {
        (Some(n), "-") => Some(-n), 
        (Some(n), _)   => Some(n), 
        _              => None,
    }
}
