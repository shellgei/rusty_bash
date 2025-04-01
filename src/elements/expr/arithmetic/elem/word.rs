//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::utils;
use crate::utils::exit;
use super::super::{ArithElem, ArithmeticExpr};
use super::{float, int};
use super::Word;

pub fn to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    if pre_increment != 0 && post_increment != 0 
    || w.text.find('\'').is_some() {
        return Err(ExecError::OperandExpected(w.text.to_string()));
    }

    let name = w.eval_as_value(core)?;

    match pre_increment {
        0 => change_variable(&name, core, post_increment, false),
        _ => change_variable(&name, core, pre_increment, true),
    }
}

fn to_num(w: &Word, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    if w.text.find('\'').is_some() {
        return Err(ExecError::OperandExpected(w.text.to_string()));
    }

    let name = w.eval_as_value(core)?;
    str_to_num(&name, core)
}

pub fn str_to_num(name: &str, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    let mut name = name.to_string();

    const RESOLVE_LIMIT: i32 = 10000;

    for i in 0..RESOLVE_LIMIT {
        match utils::is_name(&name, core) {
            true  => name = core.db.get_param(&name)?,
            false => break,
        }

        if i == RESOLVE_LIMIT - 1 {
            return Err(ExecError::Recursion(name.clone()));
        }
    }

    match single_str_to_num(&name, core) {
        Ok(e)  => Ok(e),
        Err(_) => resolve_arithmetic_op(&name, core),
    }
}

fn resolve_arithmetic_op(name: &str, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    let mut f = Feeder::new(&name);
    let mut parsed = match ArithmeticExpr::parse(&mut f, core, false, "") {
        Ok(Some(p)) => p,
        _    => return Err(ExecError::OperandExpected(name.to_string())),
    };

    if parsed.elements.len() == 1 { // In this case, the element is not changed by the evaluation.
        return Err(ExecError::OperandExpected(name.to_string()));
    }

    if let Ok(eval) = parsed.eval(core) {
        return single_str_to_num(&eval, core);
    }

    Err(ExecError::OperandExpected(name.to_string()))
}

fn single_str_to_num(name: &str, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    if name.contains('.') {
        let f = float::parse(&name)?;
        return Ok(ArithElem::Float(f));
    }

    if utils::is_name(&name, core) {
        return Ok( ArithElem::Integer(0) );
    }

    let n = int::parse(&name)?;
    Ok( ArithElem::Integer(n) )
}

fn change_variable(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<ArithElem, ExecError> {
    if ! utils::is_name(name, core) {
        return match inc != 0 && ! pre {
            true  => Err(ExecError::OperandExpected(name.to_string())),
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

pub fn substitution(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore)-> Result<(), ExecError> {
    let mut right = match stack.pop() {
        Some(mut e) => {e.change_to_value(0, core)?; e},
        _ => return Err(ExecError::OperandExpected(op.to_string())),
    };

    let left = match stack.pop() {
        Some(ArithElem::Word(w, 0)) => w,
        Some(ArithElem::Word(_, _)) => return Err(ExecError::AssignmentToNonVariable(op.to_string()) ),
        _ => return Err(ExecError::AssignmentToNonVariable(op.to_string()) ),
    };

    stack.push( subs(op, &left, &mut right, core)? );
    Ok(())
}

fn subs(op: &str, w: &Word, right_value: &mut ArithElem, core: &mut ShellCore)
                                      -> Result<ArithElem, ExecError> {
    if w.text.find('\'').is_some() {
        return Err(ExecError::OperandExpected(w.text.to_string()));
    }

    let name = w.eval_as_value(core)?;
    right_value.change_to_value(0, core)?; // InParen -> Value
    let right_str = right_value.to_string_asis();

    match op {
        "=" => {
            core.db.set_param(&name, &right_str, None)?;
            return Ok(right_value.clone());
        },
        "+=" => {
            let mut val_str = core.db.get_param(&name)?;
            if val_str == "" {
                val_str = "0".to_string();
            }
            if let Ok(left) = val_str.parse::<i64>() {
                match right_value {
                    ArithElem::Integer(n) => {
                        core.db.set_param(&name, &(left + *n).to_string(), None)?;
                        return Ok(ArithElem::Integer(left + *n));
                    },
                    _ => {},
                }
            }else if let Ok(left) = val_str.parse::<f64>() {
                if let ArithElem::Float(f) = right_value {
                    core.db.set_param(&name, &(left + *f).to_string(), None)?;
                    return Ok(ArithElem::Float(left + *f));
                }
            }
        },
        _   => {},
    }

    match (to_num(w, core)?, right_value) {
        (ArithElem::Integer(cur), ArithElem::Integer(right)) => Ok(int::substitute(op, &name, cur, *right, core)?),
        (ArithElem::Float(cur), ArithElem::Integer(right)) => Ok(float::substitute(op, &name, cur, *right as f64, core)?),
        (ArithElem::Float(cur), ArithElem::Float(right)) => Ok(float::substitute(op, &name, cur, *right, core)?),
        (ArithElem::Integer(cur), ArithElem::Float(right)) => Ok(float::substitute(op, &name, cur as f64, *right, core)?),
        _ => Err(ExecError::Other("not supported yet".to_string())),
    }
}
