//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils;
use crate::utils::{error, exit};
use super::{ArithElem, ArithmeticExpr, float, int, Word};
use crate::elements::subscript::Subscript;

pub fn to_operand(name: &String, sub: &mut Subscript, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<ArithElem, String> {
    let key = match sub.eval(core, name) {
        Some(s) => s, 
        None => return Err(format!("{}: wrong substitution", &name)),
    };

    let value = core.db.get_array(name, &key);

    if value == "" {
        return Ok( ArithElem::Integer(0) );
    }

    match value.parse::<i64>() {
        Ok(n) => return Ok( ArithElem::Integer(n) ),
        Err(_) => return Err(format!("{}: not an interger", &name)),
    }


    /*
    let res = match pre_increment {
        0 => change_variable(&name, sub, core, post_increment, false),
        _ => change_variable(&name, sub, core, pre_increment, true),
    };*/

    //return Err(error::syntax(&name));
}

/*
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

pub fn str_to_num(name: &str, sub: &mut Subscript, core: &mut ShellCore) -> Result<ArithElem, String> {

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

    if let Some(eval) = parsed.eval(core) {
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
*/

fn change_variable(name: &str, sub: &mut Subscript, core: &mut ShellCore,
                   inc: i64, pre: bool) -> Result<ArithElem, String> {
    /*
    match str_to_num(&name, sub, core) {
        Ok(ArithElem::Integer(n))        => {
            if ! core.db.set_param(name, &(n + inc).to_string()) {
                return Err(error::readonly(&name));
            }
            match pre {
                true  => Ok(ArithElem::Integer(n+inc)),
                false => Ok(ArithElem::Integer(n)),
            }
        },
        Ok(ArithElem::Float(n))        => {
            if ! core.db.set_param(name, &(n + inc as f64).to_string()) {
                return Err(error::readonly(&name));
            }
            match pre {
                true  => Ok(ArithElem::Float(n+inc as f64)),
                false => Ok(ArithElem::Float(n)),
            }
        },
        Ok(_) => exit::internal("unknown element"),
        Err(err_msg) => return Err(err_msg), 
    }
    */
    return Err(error::readonly(&name));
}

/*
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
            if ! core.db.set_param(&name, &right_str) {
                return Err(error::readonly(&name));
            }
            return Ok(right_value.clone());
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
*/
