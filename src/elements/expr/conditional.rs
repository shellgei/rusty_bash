//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod elem;
mod parser;

use crate::{Feeder, ShellCore, utils};
use crate::elements::expr::arithmetic::elem::{int, float};
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::utils::{file_check, glob};
use crate::elements::word::Word;
use regex::Regex;
use self::elem::CondElem;
use super::arithmetic::elem::ArithElem;
use std::env;

fn to_operand(w: &mut Word) -> Result<CondElem, ExecError> {
    match w.make_unquoted_word() {
        Some(v) => Ok(CondElem::Operand(v)),
        None => Ok(CondElem::Operand("".to_string())),
    }
}

fn pop_operand(stack: &mut Vec<CondElem>, core: &mut ShellCore, glob: bool) -> Result<CondElem, ExecError> {
    match stack.pop() {
        Some(CondElem::InParen(mut expr)) => expr.eval(core),
        Some(CondElem::Word(mut w)) => {
            if glob {
                let p = w.eval_for_case_pattern(core)?;
                return Ok(CondElem::Operand(p));
            }
            to_operand(&mut w)
        },
        Some(elem) => Ok(elem),
        None => return Err(ArithError::OperandExpected("".to_string()).into()),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConditionalExpr {
    pub text: String,
    elements: Vec<CondElem>,
}

impl ConditionalExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<CondElem, ExecError> {
        let mut cp = self.clone();

        let mut from = 0;
        let mut next = true;
        let mut last = CondElem::Ans(true);

        for e in cp.elements.iter_mut() {
            e.eval(core)?;
        }

        if core.db.flags.contains('x') {
            let mut elems = cp.elements.clone()
                           .into_iter().map(|e| e.to_string())
                           .collect::<Vec<String>>();
            elems.pop();
            eprintln!("{} ]]\r", &elems.join(" "));
        }

        for i in 0..cp.elements.len() {
            match cp.elements[i] {
                CondElem::And | CondElem::Or => {
                    if next {
                        last = Self::calculate(&cp.elements[from..i], core)?;
                    }
                    from = i + 1;

                    next = match (&cp.elements[i], &last) {
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
    
        match pop_operand(&mut stack, core, false) {
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
                CondElem::Not => match pop_operand(&mut stack, core, false) {
                    Ok(CondElem::Ans(res)) => {
                        stack.push(CondElem::Ans(!res));
                        Ok(())
                    },
                    Ok(CondElem::Operand(s)) => {
                        stack.push(CondElem::Ans(s == ""));
                        Ok(())
                    },
                    _ => Err(ExecError::Other("no operand to negate".to_string())),
                },
               // _ => Err(ExecError::Other( error::syntax("TODO"))),
                _ => Err(ArithError::OperandExpected("TODO".to_string()).into()),
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
        let operand = match pop_operand(stack, core, false) {
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
        let right = match pop_operand(stack, core, false) {
            Ok(CondElem::Regex(right)) => right, 
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };

        let left = match pop_operand(stack, core, false) {
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

        core.db.set_array("BASH_REMATCH", vec![], None)?;
        if let Some(res) = re.captures(&left) {
            for i in 0.. {
                if let Some(e) = res.get(i) {
                    let s = e.as_str().to_string();
                    core.db.set_array_elem("BASH_REMATCH", &s, i, None)?;
                }else{
                    break;
                }
            }
            stack.push( CondElem::Ans(true) );
        }else{
            stack.push( CondElem::Ans(false) );
        }

        return Ok(());
    }

    fn resolve_arithmetic_op(name: &str, core: &mut ShellCore) -> Result<ArithElem, ArithError> {
        let mut f = Feeder::new(&name);
        let mut parsed = match ArithmeticExpr::parse(&mut f, core, false, "") {
            Ok(Some(p)) => p,
            _    => return Err(ArithError::OperandExpected(name.to_string())),
        };

        if let Ok(eval) = parsed.eval(core) {
            return Self::single_str_to_num(&eval, core);
        }
    
        Err(ArithError::OperandExpected(name.to_string()))
    }

    fn single_str_to_num(name: &str, core: &mut ShellCore) -> Result<ArithElem, ArithError> {
        if name.contains('.') {
            let f = float::parse(&name)?;
            return Ok(ArithElem::Float(f, None));
        }
    
        if utils::is_name(&name, core) {
            return Ok( ArithElem::Integer(0, None) );
        }
    
        let n = int::parse(&name)?;
        Ok( ArithElem::Integer(n, None) )
    }

    fn bin_operation(op: &str, stack: &mut Vec<CondElem>,
                     core: &mut ShellCore) -> Result<(), ExecError> {
        let right = match pop_operand(stack, core, true) {
            Ok(CondElem::Operand(name)) => name,
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };
    
        let left = match pop_operand(stack, core, false) {
            Ok(CondElem::Operand(name)) => name,
            Ok(_)  => return Err(ExecError::Other("Invalid operand".to_string())),
            Err(e) => return Err(e),
        };

        let extglob = core.shopts.query("extglob");
        if op.starts_with("=") || op == "!=" || op == "<" || op == ">" {
            let ans = match op {
                "==" | "=" => glob::parse_and_compare(&left, &right, extglob),
                "!="       => ! glob::parse_and_compare(&left, &right, extglob),
                ">"        => left > right,
                "<"        => left < right,
                _    => false,
            };

            stack.push( CondElem::Ans(ans) );
            return Ok(());
        }

        if op == "-eq" || op == "-ne" || op == "-lt" || op == "-le" || op == "-gt" || op == "-ge" {
            let lnum = match Self::resolve_arithmetic_op(&left, core)? {
                ArithElem::Integer(n, _) => n,
                _ => return Err(ExecError::Other("non integer number is not supported".to_string())),
            };
            let rnum = match Self::resolve_arithmetic_op(&right, core)? {
                ArithElem::Integer(n, _) => n,
                _ => return Err(ExecError::Other("non integer number is not supported".to_string())),
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
