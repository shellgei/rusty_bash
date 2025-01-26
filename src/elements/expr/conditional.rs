//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod elem;
mod parser;

use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::utils::{file_check, glob};
use crate::elements::word::Word;
use regex::Regex;
use self::elem::CondElem;
use super::arithmetic::word;
use super::arithmetic::elem::ArithElem;
use std::env;

fn to_operand(w: &Word, core: &mut ShellCore) -> Result<CondElem, ExecError> {
    match w.eval_for_case_pattern(core) {
        Some(v) => Ok(CondElem::Operand(v)),
        None => return Err(ExecError::Other(format!("{}: wrong substitution", &w.text))),
    }
}

fn pop_operand(stack: &mut Vec<CondElem>, core: &mut ShellCore) -> Result<CondElem, ExecError> {
    match stack.pop() {
        Some(CondElem::InParen(mut expr)) => expr.eval(core),
        Some(CondElem::Word(w)) => to_operand(&w, core),
        Some(elem) => Ok(elem),
        None => return Err(ExecError::OperandExpected("".to_string())),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConditionalExpr {
    pub text: String,
    elements: Vec<CondElem>,
}

impl ConditionalExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<CondElem, ExecError> {
        let mut from = 0;
        let mut next = true;
        let mut last = CondElem::Ans(true);
        for i in 0..self.elements.len() {
            match self.elements[i] {
                CondElem::And | CondElem::Or => {
                    if next {
                        last = Self::calculate(&self.elements[from..i], core)?;
                    }
                    from = i + 1;

                    next = match (&self.elements[i], &last) {
                        (CondElem::And, CondElem::Ans(ans)) => *ans,
                        (CondElem::Or, CondElem::Ans(ans))  => !ans,
                        _ => return Err(ExecError::Other("Internal error conditional.rs:55".to_string())),
                    };
                },
                _ => {},
            }
        }
 
        Ok(last)
    }

    fn calculate(elems: &[CondElem], core: &mut ShellCore) -> Result<CondElem, ExecError> {
        let rev_pol = Self::rev_polish(elems)?;
        let mut stack = Self::reduce(&rev_pol, core)?;
    
        match pop_operand(&mut stack, core) {
            Ok(CondElem::Operand(s))  => Ok(CondElem::Ans(s.len() > 0)), //for [[ string ]]
            other_ans             => other_ans,
        }
    }

    fn rev_polish(elems: &[CondElem]) -> Result<Vec<CondElem>, ExecError> {
        let mut ans = vec![];
        let mut stack = vec![];
    
        for e in elems {
            let ok = match e {
                CondElem::Word(_) 
                | CondElem::InParen(_) 
                | CondElem::Regex(_) => {ans.push(e.clone()); true},
                op               => Self::rev_polish_op(&op, &mut stack, &mut ans),
            };
    
            if !ok {
                return Err(ExecError::SyntaxError(e.to_string()));
            }
        }
    
        while stack.len() > 0 {
            ans.push(stack.pop().unwrap());
        }
    
        Ok(ans)
    }

    fn reduce(rev_pol: &[CondElem], core: &mut ShellCore) -> Result<Vec<CondElem>, ExecError> {
        let mut stack = vec![];

        for e in rev_pol {
            let result = match e { 
                CondElem::Word(_) | CondElem::Regex(_) | CondElem::InParen(_) => {
                    stack.push(e.clone());
                    Ok(())
                },
                CondElem::UnaryOp(ref op) => Self::unary_operation(&op, &mut stack, core),
                CondElem::BinaryOp(ref op) => {
                    if stack.is_empty() {
                        return Ok(vec![CondElem::Ans(true)]); //for [[ -ot ]] [[ == ]] [[ = ]] ...
                    }
                    if op == "=~" {
                        Self::regex_operation(&mut stack, core)
                    }else{
                        Self::bin_operation(&op, &mut stack, core)
                    }
                },
                CondElem::Not => match pop_operand(&mut stack, core) {
                    Ok(CondElem::Ans(res)) => {
                        stack.push(CondElem::Ans(!res));
                        Ok(())
                    },
                    _ => Err(ExecError::Other("no operand to negate".to_string())),
                },
               // _ => Err(ExecError::Other( error::syntax("TODO"))),
                _ => Err(ExecError::OperandExpected("TODO".to_string())),
            };
    
            if let Err(err_msg) = result {
                core.db.exit_status = 2;
                return Err(err_msg);
            }
        }

        if stack.len() != 1 { 
            let mut err = "syntax error".to_string();
            if stack.len() > 1 {
                err = format!("syntax error in conditional expression: unexpected token `{}'", &stack[0].to_string());
                ExecError::Other(err).print(core);
                err = format!("syntax error near `{}'", &stack[0].to_string());
            }
            return Err(ExecError::Other(err));
        }   

        Ok(stack)
    }

    fn unary_operation(op: &str, stack: &mut Vec<CondElem>, core: &mut ShellCore) -> Result<(), ExecError> {
        let operand = match pop_operand(stack, core) {
            Ok(CondElem::Operand(v))  => v,
            Ok(_)  => return Err(ExecError::Other("unknown operand".to_string())), 
            Err(e) => return Err(e),
        };

        if op == "-o" || op == "-v" || op == "-z" || op == "-n" {
            let ans = match op {
                "-o" => core.options.query(&operand),
                //"-v" => core.db.get_value(&operand).is_some() || env::var(&operand).is_ok(),
                "-v" => core.db.has_value(&operand) || env::var(&operand).is_ok(),
                "-z" => operand.is_empty(),
                "-n" => operand.len() > 0,
                _    => false,
            };

            stack.push( CondElem::Ans(ans) );
            return Ok(());
        }

        Self::unary_file_check(op, &operand, stack)
    }

    fn regex_operation(stack: &mut Vec<CondElem>, core: &mut ShellCore) -> Result<(), ExecError> {
        let right = match pop_operand(stack, core) {
            Ok(CondElem::Regex(right)) => right, 
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };

        let left = match pop_operand(stack, core) {
            Ok(CondElem::Operand(name)) => name,
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };

        let right_eval = match right.eval_for_regex(core) {
            Some(r) => r,
            None  => return Err(ExecError::Other("Invalid operand".to_string())),
        };

        let re = match Regex::new(&right_eval) {
            Ok(regex) => regex,
            Err(e) => return Err(ExecError::Other(e.to_string())),
        };

        stack.push( CondElem::Ans(re.is_match(&left)) );
        return Ok(());
    }

    fn bin_operation(op: &str, stack: &mut Vec<CondElem>,
                     core: &mut ShellCore) -> Result<(), ExecError> {
        let right = match pop_operand(stack, core) {
            Ok(CondElem::Operand(name)) => name,
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };
    
        let left = match pop_operand(stack, core) {
            Ok(CondElem::Operand(name)) => name,
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };

        let extglob = core.shopts.query("extglob");
        if op.starts_with("=") || op == "!=" || op == "<" || op == ">" {
            let ans = match op {
                "==" | "=" => glob::parse_and_compare(&left, &right, extglob),
                "=~"       => glob::parse_and_compare(&left, &right, extglob),
                "!="       => ! glob::parse_and_compare(&left, &right, extglob),
                ">"        => left > right,
                "<"        => left < right,
                _    => false,
            };

            stack.push( CondElem::Ans(ans) );
            return Ok(());
        }

        if op == "-eq" || op == "-ne" || op == "-lt" || op == "-le" || op == "-gt" || op == "-ge" {
            let lnum = match word::str_to_num(&left, core) {
                Ok(ArithElem::Integer(n)) => n,
                Ok(_) => return Err(ExecError::Other("non integer number is not supported".to_string())),
                Err(msg) => return Err(msg),
            };
            let rnum = match word::str_to_num(&right, core) {
                Ok(ArithElem::Integer(n)) => n,
                Ok(_) => return Err(ExecError::Other("non integer number is not supported".to_string())),
                Err(msg) => return Err(msg),
            };

            let ans = match op {
                "-eq" => lnum == rnum,
                "-ne" => lnum != rnum,
                "-lt" => lnum < rnum,
                "-le" => lnum <= rnum,
                "-gt" => lnum > rnum,
                "-ge" => lnum >= rnum,
                _    => false,
            };

            stack.push( CondElem::Ans(ans) );
            return Ok(());
        }

        let result = file_check::metadata_comp(&left, &right, op);
        stack.push( CondElem::Ans(result) );
        Ok(())
    }

    fn unary_file_check(op: &str, s: &String, stack: &mut Vec<CondElem>) -> Result<(), ExecError> {
        let result = match op {
            "-a" | "-e"  => file_check::exists(s),
            "-d"  => file_check::is_dir(s),
            "-f"  => file_check::is_regular_file(s),
            "-h" | "-L"  => file_check::is_symlink(s),
            "-r"  => file_check::is_readable(s),
            "-t"  => file_check::is_tty(s),
            "-w"  => file_check::is_writable(s),
            "-x"  => file_check::is_executable(s),
            "-b" | "-c" | "-g" | "-k" | "-p" | "-s" | "-u" | "-G" | "-N" | "-O" | "-S"
                  => file_check::metadata_check(s, op),
            _  => return Err(ExecError::Other("unsupported option".to_string())),
        };

        stack.push( CondElem::Ans(result) );
        Ok(())
    }

    fn rev_polish_op(elem: &CondElem,
                     stack: &mut Vec<CondElem>, ans: &mut Vec<CondElem>) -> bool {
        loop {
            match stack.last() {
                None => {
                    stack.push(elem.clone());
                    break;
                },
                Some(_) => {
                    let last = stack.last().unwrap();
                    if last.order() <= elem.order() {
                        stack.push(elem.clone());
                        break;
                    }
                    ans.push(stack.pop().unwrap());
                },
            }
        }
    
        true
    }
}
