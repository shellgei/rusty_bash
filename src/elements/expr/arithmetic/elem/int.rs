//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::variable;
use super::ArithElem;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::utils::exit;
use crate::ShellCore;

pub fn unary_calc(op: &str, num: i128, stack: &mut Vec<ArithElem>) -> Result<(), ExecError> {
    match op {
        "+" => stack.push(ArithElem::Integer(num)),
        "-" => stack.push(ArithElem::Integer(-num)),
        "!" => stack.push(ArithElem::Integer(if num == 0 { 1 } else { 0 })),
        "~" => stack.push(ArithElem::Integer(!num)),
        _ => exit::internal("unknown unary operator"),
    }
    Ok(())
}

pub fn bin_calc(
    op: &str,
    left: i128,
    right: i128,
    stack: &mut Vec<ArithElem>,
) -> Result<(), ArithError> {
    let bool_to_01 = |b| {
        if b {
            1
        } else {
            0
        }
    };

    let ans = match op {
        "+" => left + right,
        "-" => left - right,
        "*" => left * right,
        "&" => left & right,
        "^" => left ^ right,
        "|" => left | right,
        "&&" => bool_to_01(left != 0 && right != 0),
        "||" => bool_to_01(left != 0 || right != 0),
        "<<" => {
            if right < 0 {
                0
            } else {
                left << right
            }
        }
        ">>" => {
            if right < 0 {
                0
            } else {
                left >> right
            }
        }
        "<=" => bool_to_01(left <= right),
        ">=" => bool_to_01(left >= right),
        "<" => bool_to_01(left < right),
        ">" => bool_to_01(left > right),
        "==" => bool_to_01(left == right),
        "!=" => bool_to_01(left != right),
        "%" | "/" => {
            if right == 0 {
                return Err(ArithError::DivZero(right.to_string()));
            }
            match op {
                "%" => left % right,
                _ => left / right,
            }
        }
        "**" => {
            if right >= 0 {
                let r = right.try_into().unwrap();
                left.pow(r)
            } else {
                return Err(ArithError::Exponent(right));
            }
        }
        _ => exit::internal("unknown binary operator"),
    };

    stack.push(ArithElem::Integer(ans));
    Ok(())
}

pub fn substitute(
    op: &str,
    name: &str,
    index: &str,
    cur: i128,
    right: i128,
    core: &mut ShellCore,
) -> Result<ArithElem, ExecError> {
    let new_value = match op {
        "+=" => cur + right,
        "-=" => cur - right,
        "*=" => cur * right,
        "&=" => cur & right,
        "^=" => cur ^ right,
        "|=" => cur | right,
        "<<=" => {
            if right < 0 {
                0
            } else {
                cur << right
            }
        }
        ">>=" => {
            if right < 0 {
                0
            } else {
                cur >> right
            }
        }
        "/=" | "%=" => {
            if right == 0 {
                return Err(ArithError::DivZero(right.to_string()).into());
            }
            match op == "%=" {
                true => cur % right,
                false => cur / right,
            }
        }
        _ => return Err(ArithError::OperandExpected(op.to_string()).into()),
    };

    core.db
        .set_param2(name, index, &new_value.to_string(), None)?;
    Ok(ArithElem::Integer(new_value))
}

fn parse_with_base(base: i128, s: &mut str, org: &str) -> Result<i128, ArithError> {
    if s.is_empty() {
        return Err(ArithError::InvalidIntConst(org.to_string()));
    }

    let mut ans = 0;
    for ch in s.chars() {
        ans *= base;
        let num = if ch.is_ascii_digit() {
            ch as i128 - '0' as i128
        } else if ch.is_ascii_lowercase() {
            ch as i128 - 'a' as i128 + 10
        } else if ch.is_ascii_uppercase() {
            match base <= 36 {
                true => ch as i128 - 'A' as i128 + 10,
                false => ch as i128 - 'A' as i128 + 36,
            }
        } else if ch == '@' {
            62
        } else if ch == '_' {
            63
        } else {
            return Err(ArithError::InvalidNumber(org.to_string()));
        };

        match num < base {
            true => ans += num,
            false => return Err(ArithError::ValueTooGreatForBase(org.to_string())),
        }
    }

    Ok(ans)
}

fn get_base(s: &mut String) -> Result<i128, ArithError> {
    let s_org = s.to_string();
    if s.starts_with("0x") || s.starts_with("0X") {
        s.remove(0);
        s.remove(0);
        return Ok(16);
    }

    if s.starts_with("0") && s.len() > 1 {
        if s.contains('#') {
            return Err(ArithError::InvalidNumber(s.clone()));
        }
        s.remove(0);
        return Ok(8);
    }

    if let Some(n) = s.find("#") {
        let base_str = s[..n].to_string();
        *s = s[(n + 1)..].to_string();
        return match base_str.parse::<i128>() {
            Ok(n) => match n <= 64 {
                true => Ok(n),
                false => Err(ArithError::InvalidBase(s_org.to_string())),
            },
            _ => Err(ArithError::InvalidBase(s_org.to_string())),
        };
    }

    Ok(10)
}

pub fn parse(s: &str) -> Result<i128, ArithError> {
    if s.find('\'').is_some() || s.find('.').is_some() {
        return Err(ArithError::InvalidNumber(s.to_string()));
    }
    if s.is_empty() {
        return Ok(0);
    }

    let mut sw = s.to_string();
    let sign = variable::get_sign(&mut sw);
    let base = get_base(&mut sw)?;
    let n = parse_with_base(base, &mut sw, s)?;

    match sign.as_str() {
        "-" => Ok(-n),
        _ => Ok(n),
    }
}
