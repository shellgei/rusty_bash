//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::Elem;
use super::calculator::exponent_error_msg;

pub fn bin_calc(op: &str, left: i64, right: i64, stack: &mut Vec<Elem>) -> Result<(), String> {
    let bool_to_01 = |b| { if b { 1 } else { 0 } };

    let ans = match op {
        "+"  => left + right,
        "-"  => left - right,
        "*"  => left * right,
        "&"  => left & right,
        "^"  => left ^ right,
        "|"  => left | right,
        "&&"  => bool_to_01( left != 0 && right != 0 ),
        "||"  => bool_to_01( left != 0 || right != 0 ),
        "<<"  => if right < 0 {0} else {left << right},
        ">>"  => if right < 0 {0} else {left >> right},
        "<="  => bool_to_01( left <= right ),
        ">="  => bool_to_01( left >= right ),
        "<"  => bool_to_01( left < right ),
        ">"  => bool_to_01( left > right ),
        "=="  => bool_to_01( left == right ),
        "!="  => bool_to_01( left != right ),
        "%" | "/" => {
            if right == 0 {
                return Err("divided by 0".to_string());
            }
            match op {
                "%" => left % right,
                _   => left / right,
            }
        },
        "**" => {
            if right >= 0 {
                let r = right.try_into().unwrap();
                left.pow(r)
            }else{
                return Err( exponent_error_msg(&right.to_string()) );
            }
        },
        _    => panic!("SUSH INTERNAL ERROR: unknown binary operator"),
    };

    stack.push(Elem::Integer(ans));
    Ok(())
}

pub fn substitute(op: &str, name: &String, cur: i64, right: i64, core: &mut ShellCore)
                                      -> Result<Elem, String> {
    let new_value = match op {
        "+=" => cur + right,
        "-=" => cur - right,
        "*=" => cur * right,
        "&="  => cur & right,
        "^="  => cur ^ right,
        "|="  => cur | right,
        "<<="  => if right < 0 {0} else {cur << right},
        ">>="  => if right < 0 {0} else {cur >> right},
        "/=" | "%=" => {
            if right == 0 {
                return Err("divided by 0".to_string());
            }
            match op == "%=" {
                true  => cur % right,
                false => cur / right,
            }
        },
        _   => return Err("Not supprted operation for integer numbers".to_string()),
    };

    core.data.set_param(&name, &new_value.to_string());
    Ok(Elem::Integer(new_value))
}

pub fn parse_with_base(base: i64, s: &mut String) -> Option<i64> {
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

