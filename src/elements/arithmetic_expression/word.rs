//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder};
use super::{Elem, float, int};
use crate::elements::arithmetic_expression::Word;

pub fn to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<Elem, String> {
    if pre_increment != 0 && post_increment != 0 
    || w.text.find('\'').is_some() {
        return Err(error_message::syntax(&w.text));
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
        Ok(n)  => return Ok(n),
        Err(e) => return Err(e),
    }
}

fn to_num(w: &Word, core: &mut ShellCore) -> Result<Elem, String> {
    if w.text.find('\'').is_some() {
        return Err(error_message::syntax(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    str_to_num(&name, core)
}

fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

fn str_to_num(name: &str, core: &mut ShellCore) -> Result<Elem, String> {
    let mut name = name.to_string();

    const RESOLVE_LIMIT: i32 = 10000;

    for i in 0..RESOLVE_LIMIT {
        match is_name(&name, core) {
            true  => name = core.data.get_param(&name),
            false => break,
        }

        if i == RESOLVE_LIMIT - 1 {
            return Err(error_message::recursion(&name));
        }
    }

    if let Some(n) = int::parse(&name) {
        Ok( Elem::Integer(n) )
    }else if is_name(&name, core) {
        Ok( Elem::Integer(0) )
    } else if let Some(f) = float::parse(&name) {
        Ok( Elem::Float(f) )
    }else{
        Err(error_message::syntax(&name))
    }
}

fn change_variable(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<Elem, String> {
    if ! is_name(name, core) {
        return match inc != 0 && ! pre {
            true  => Err(error_message::syntax(name)),
            false => str_to_num(&name, core),
        }
    }

    match str_to_num(&name, core) {
        Ok(Elem::Integer(n))        => {
            core.data.set_param(name, &(n + inc).to_string());
            match pre {
                true  => Ok(Elem::Integer(n+inc)),
                false => Ok(Elem::Integer(n)),
            }
        },
        Ok(Elem::Float(n))        => {
            core.data.set_param(name, &(n + inc as f64).to_string());
            match pre {
                true  => Ok(Elem::Float(n+inc as f64)),
                false => Ok(Elem::Float(n)),
            }
        },
        Ok(_) => panic!("SUSH INTERNAL ERROR: unknown element"),
        Err(err_msg) => return Err(err_msg), 
    }
}

pub fn get_sign(s: &mut String) -> String {
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

pub fn substitution(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore)-> Result<(), String> {
    let right = match stack.pop() {
        Some(e) => e,
        _       => return Err( error_message::syntax(op) ),
    };

    let left = match stack.pop() {
        Some(Elem::Word(w, 0)) => w,
        Some(Elem::Word(_, _)) => return Err( error_message::assignment(op) ),
        _ => return Err( error_message::assignment(op) ),
    };

    match subs(op, &left, &right, core) {
        Ok(elem) => stack.push(elem),
        Err(msg) => return Err(msg),
    }
    Ok(())
}

fn subs(op: &str, w: &Word, right_value: &Elem, core: &mut ShellCore)
                                      -> Result<Elem, String> {
    if w.text.find('\'').is_some() {
        return Err(error_message::syntax(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    let right_str = match right_value {
        Elem::Integer(n) => n.to_string(),
        Elem::Float(f)   => f.to_string(),
        _ => panic!("SUSH INTERNAL ERROR: not a value"),
    };

    match op {
        "=" => {
            core.data.set_param(&name, &right_str);
            return Ok(right_value.clone());
        },
        _   => {},
    }

    let current_num = match to_num(w, core) {
        Ok(n)  => n,
        Err(e) => return Err(e),
    };

    match (current_num, right_value) {
        (Elem::Integer(cur), Elem::Integer(right)) => int::substitute(op, &name, cur, *right, core),
        (Elem::Float(cur), Elem::Integer(right)) => float::substitute(op, &name, cur, *right as f64, core),
        (Elem::Float(cur), Elem::Float(right)) => float::substitute(op, &name, cur, *right, core),
        (Elem::Integer(cur), Elem::Float(right)) => float::substitute(op, &name, cur as f64, *right, core),
        _ => Err("support not yet".to_string()),
    }

}
