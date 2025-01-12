//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils;
use crate::utils::{error, exit};
use super::{ArithElem, ArithmeticExpr, float, int, Word};

pub fn to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<ArithElem, String> {
    if pre_increment != 0 && post_increment != 0 
    || w.text.find('\'').is_some() {
        return Err(error::syntax(&w.text));
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

fn to_num(w: &Word, core: &mut ShellCore) -> Result<ArithElem, String> {
    if w.text.find('\'').is_some() {
        return Err(error::syntax(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    str_to_num(&name, core)
}

pub fn str_to_num(name: &str, core: &mut ShellCore) -> Result<ArithElem, String> {
    let mut name = name.to_string();

    const RESOLVE_LIMIT: i32 = 10000;

    for i in 0..RESOLVE_LIMIT {
        match utils::is_name(&name, core) {
            true  => name = core.db.get_param(&name)?,
            false => break,
        }

        if i == RESOLVE_LIMIT - 1 {
            return Err(error::recursion(&name));
        }
    }

    match single_str_to_num(&name, core) {
        Some(e) => Ok(e),
        None    => resolve_arithmetic_op(&name, core),
    }
}

fn resolve_arithmetic_op(name: &str, core: &mut ShellCore) -> Result<ArithElem, String> {
    let mut f = Feeder::new(&name);
    let mut parsed = match ArithmeticExpr::parse(&mut f, core, false) {
        Some(p) => p,
        None    => return Err(error::syntax(&name)),
    };

    if parsed.elements.len() == 1 { // In this case, the element is not changed by the evaluation.
        return Err(error::syntax(&name));
    }

    if let Ok(eval) = parsed.eval(core) {
        if let Some(e) = single_str_to_num(&eval, core) {
            return Ok(e);
        }
    }

    Err(error::syntax(&name))
}

fn single_str_to_num(name: &str, core: &mut ShellCore) -> Option<ArithElem> {
    if let Some(n) = int::parse(&name) {         Some( ArithElem::Integer(n) )
    }else if utils::is_name(&name, core) {       Some( ArithElem::Integer(0) )
    }else if let Some(f) = float::parse(&name) { Some( ArithElem::Float(f) )
    }else{                                       None }
}

fn change_variable(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<ArithElem, String> {
    if ! utils::is_name(name, core) {
        return match inc != 0 && ! pre {
            true  => Err(error::syntax(name)),
            false => str_to_num(&name, core),
        }
    }

    match str_to_num(&name, core) {
        Ok(ArithElem::Integer(n))        => {
            core.db.set_param(name, &(n + inc).to_string(), None)?;
            match pre {
                true  => Ok(ArithElem::Integer(n+inc)),
                false => Ok(ArithElem::Integer(n)),
            }
        },
        Ok(ArithElem::Float(n))        => {
            core.db.set_param(name, &(n + inc as f64).to_string(), None)?;
            match pre {
                true  => Ok(ArithElem::Float(n+inc as f64)),
                false => Ok(ArithElem::Float(n)),
            }
        },
        Ok(_) => exit::internal("unknown element"),
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

pub fn substitution(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore)-> Result<(), String> {
    let right = match stack.pop() {
        Some(e) => e,
        _       => return Err( error::syntax(op) ),
    };

    let left = match stack.pop() {
        Some(ArithElem::Word(w, 0)) => w,
        Some(ArithElem::Word(_, _)) => return Err( error::assignment(op) ),
        _ => return Err( error::assignment(op) ),
    };

    match subs(op, &left, &right, core) {
        Ok(elem) => stack.push(elem),
        Err(msg) => return Err(msg),
    }
    Ok(())
}

fn subs(op: &str, w: &Word, right_value: &ArithElem, core: &mut ShellCore)
                                      -> Result<ArithElem, String> {
    if w.text.find('\'').is_some() {
        return Err(error::syntax(&w.text));
    }

    let name = match w.eval_as_value(core) {
        Some(v) => v, 
        None => return Err(format!("{}: wrong substitution", &w.text)),
    };

    let right_str = match right_value {
        ArithElem::Integer(n) => n.to_string(),
        ArithElem::Float(f)   => f.to_string(),
        _ => exit::internal("not a value"),
    };

    match op {
        "=" => {
            core.db.set_param(&name, &right_str, None)?;
            return Ok(right_value.clone());
        },
        "+=" => {
            let val_str = core.db.get_param(&name)?;
            if let Ok(left) = val_str.parse::<i64>() {
                if let ArithElem::Integer(n) = right_value {
                    core.db.set_param(&name, &(left + n).to_string(), None)?;
                    return Ok(ArithElem::Integer(left + n));
                }
            }else if let Ok(left) = val_str.parse::<f64>() {
                if let ArithElem::Float(f) = right_value {
                    core.db.set_param(&name, &(left + f).to_string(), None)?;
                    return Ok(ArithElem::Float(left + f));
                }
            }
        },
        _   => {},
    }

    let current_num = match to_num(w, core) {
        Ok(n)  => n,
        Err(e) => return Err(e),
    };

    match (current_num, right_value) {
        (ArithElem::Integer(cur), ArithElem::Integer(right)) => int::substitute(op, &name, cur, *right, core),
        (ArithElem::Float(cur), ArithElem::Integer(right)) => float::substitute(op, &name, cur, *right as f64, core),
        (ArithElem::Float(cur), ArithElem::Float(right)) => float::substitute(op, &name, cur, *right, core),
        (ArithElem::Integer(cur), ArithElem::Float(right)) => float::substitute(op, &name, cur as f64, *right, core),
        _ => Err("support not yet".to_string()),
    }
}
