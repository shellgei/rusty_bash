//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder};
use crate::utils::file_check;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
pub enum Elem {
    UnaryOp(String),
    BinaryOp(String),
    Word(Word),
    Operand(String),
    InParen(ConditionalExpr),
    Not, // !
    And, // &&
    Or, // ||
    Ans(bool),
}

fn op_order(op: &Elem) -> u8 {
    match op {
        Elem::UnaryOp(_) => 14,
        Elem::BinaryOp(_) => 13,
        Elem::Not => 12,
        //Elem::And | Elem::Or => 12,
        _ => 0,
    }
}

pub fn to_string(op: &Elem) -> String {
    match op {
        Elem::UnaryOp(op) => op.to_string(),
        Elem::BinaryOp(op) => op.to_string(),
        Elem::InParen(expr) => expr.text.clone(),
        Elem::Word(w) => w.text.clone(),
        Elem::Operand(op) => op.to_string(),
        Elem::Not => "!".to_string(),
        Elem::And => "&&".to_string(),
        Elem::Or => "||".to_string(),
        Elem::Ans(true) => "true".to_string(),
        Elem::Ans(false) => "false".to_string(),
    }
}

fn to_operand(w: &Word, core: &mut ShellCore) -> Result<Elem, String> {
    match w.eval_as_value(core) {
        Some(v) => Ok(Elem::Operand(v)),
        None => return Err(format!("{}: wrong substitution", &w.text)),
    }
}

fn pop_operand(stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    match stack.pop() {
        Some(Elem::InParen(mut expr)) => expr.eval(core),
        Some(Elem::Word(w)) => to_operand(&w, core),
        Some(elem) => Ok(elem),
        None => return Err("no operand".to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr {
    pub text: String,
    elements: Vec<Elem>,
}

impl ConditionalExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<Elem, String> {

        let mut from = 0;
        let mut next = true;
        let mut last = Elem::Ans(true);
        for i in 0..self.elements.len() {
            match self.elements[i] {
                Elem::And | Elem::Or => {
                    if next {
                        last = match Self::calculate(&self.elements[from..i], core) {
                            Ok(elem) => elem, 
                            Err(e)   => return Err(e),
                        };
                    }
                    from = i + 1;
                    next = match (&self.elements[i], &last) {
                        (Elem::And, Elem::Ans(ans)) => *ans,
                        (Elem::Or, Elem::Ans(ans))  => !ans,
                        _ => panic!("SUSH INTERNAL ERROR"),
                    };
                },
                _ => {},
            }
        }

        if next { 
            Self::calculate(&self.elements[from..], core)
        }else {
            Ok(last)
        }
    }

    fn calculate(elems: &[Elem], core: &mut ShellCore) -> Result<Elem, String> {
        let rev_pol = match Self::rev_polish(elems) {
            Ok(ans) => ans,
            Err(e) => return Err(e),
        };
        let mut stack = match Self::reduce(&rev_pol, core) {
            Ok(s)  => s, 
            Err(e) => return Err(e),
        };
    
        pop_operand(&mut stack, core)
    }

    fn rev_polish(elems: &[Elem]) -> Result<Vec<Elem>, String> {
        let mut ans = vec![];
        let mut stack = vec![];
    
        for e in elems {
            let ok = match e {
                Elem::Word(_) | Elem::InParen(_) => {ans.push(e.clone()); true},
                op               => Self::rev_polish_op(&op, &mut stack, &mut ans),
            };
    
            if !ok {
                let msg = "syntax error near ".to_owned() + &to_string(e);
                return Err(msg);
            }
        }
    
        while stack.len() > 0 {
            ans.push(stack.pop().unwrap());
        }
    
        Ok(ans)
    }

    fn reduce(rev_pol: &[Elem], core: &mut ShellCore) -> Result<Vec<Elem>, String> {
        let mut stack = vec![];

        for e in rev_pol {
            let result = match e { 
                Elem::Word(_) | Elem::InParen(_) => {
                    stack.push(e.clone());
                    Ok(())
                },
                Elem::UnaryOp(ref op)  => {
                    Self::unary_operation(&op, &mut stack, core)
                },
                Elem::BinaryOp(ref op)  => {
                    Self::bin_operation(&op, &mut stack, core)
                },
                Elem::Not => match pop_operand(&mut stack, core) {
                    Ok(Elem::Ans(res)) => {
                        stack.push(Elem::Ans(!res));
                        Ok(())
                    },
                    _ => Err("no operand to negate".to_string()),
                },
                _ => Err( error_message::syntax("TODO")),
            };
    
            if let Err(err_msg) = result {
                core.data.set_param("?", "2");
                return Err(err_msg);
            }
        }

        if stack.len() != 1 { 
            let mut err = "syntax error".to_string();
            if stack.len() > 1 {
                err = error_message::syntax_in_cond_expr(&to_string(&stack[0]));
                error_message::print(&err, core, true);
                err = format!("syntax error near `{}'", to_string(&stack[0]));
            }
            return Err(err);
        }   

        Ok(stack)
    }

    fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
        let operand = match pop_operand(stack, core) {
            Ok(Elem::Operand(v))  => v,
            Ok(_)  => return Err("unknown operand".to_string()), 
            Err(e) => return Err(e + " to conditional unary operator"),
        };

        if op == "-o" {
            let ans = core.options.query(&operand) || core.shopts.query(&operand);
            stack.push( Elem::Ans(ans) );
            return Ok(());
        }

        Self::unary_file_check(op, &operand, stack)
    }

    fn bin_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
        let right = match pop_operand(stack, core) {
            Ok(Elem::Operand(name)) => name,
            Ok(_)  => return Err("Invalid operand".to_string()),
            Err(e) => return Err(e),
        };
    
        let left = match pop_operand(stack, core) {
            Ok(Elem::Operand(name)) => name,
            Ok(_)  => return Err("Invalid operand".to_string()),
            Err(e) => return Err(e),
        };

        let result = file_check::metadata_comp(&left, &right, op);
        stack.push( Elem::Ans(result) );
        Ok(())
    }

    fn unary_file_check(op: &str, s: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
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
            _  => return Err("unsupported option".to_string()),
        };

        stack.push( Elem::Ans(result) );
        Ok(())
    }

    fn rev_polish_op(elem: &Elem,
                     stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
        loop {
            match stack.last() {
                None => {
                    stack.push(elem.clone());
                    break;
                },
                Some(_) => {
                    let last = stack.last().unwrap();
                    if op_order(last) <= op_order(elem) {
                        stack.push(elem.clone());
                        break;
                    }
                    ans.push(stack.pop().unwrap());
                },
            }
        }
    
        true
    }

    fn new() -> ConditionalExpr {
        ConditionalExpr {
            text: String::new(),
            elements: vec![],
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with("]]")
        || feeder.starts_with(")")
        || feeder.starts_with("(") {
            return false;
        }

        match Word::parse(feeder, core, false) {
            Some(w) => {
                ans.text += &w.text.clone();
                ans.elements.push(Elem::Word(w));

                true
            },
            _ => false
        }
    }

    fn eat_compare_op(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_compare_op(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(Elem::BinaryOp(opt));

        true
    }

    fn eat_file_check_option(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_check_option(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(Elem::UnaryOp(opt));

        true
    }

    fn eat_not_and_or(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("!") {
            ans.text += &feeder.consume(1);
            ans.elements.push( Elem::Not );
            return true;
        }
        if feeder.starts_with("&&") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::And );
            return true;
        }
        if feeder.starts_with("||") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::Or );
            return true;
        }

        false
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(e) = ans.elements.last() {
            match e {
                Elem::UnaryOp(_) => {
                    return false
                },
                _ => {},
            }
        }

        if ! feeder.starts_with("(") {
            return false;
        }

        ans.text += &feeder.consume(1);

        let expr = match Self::parse(feeder, core) {
            Some(e) => e,
            None    => return false,
        };

        if ! feeder.starts_with(")") {
            return false;
        }

        ans.text += &expr.text.clone();
        ans.elements.push( Elem::InParen(expr) );
        ans.text += &feeder.consume(1);
        true
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.scanner_blank(core) {
            0 => false,
            n => {
                ans.text += &feeder.consume(n);
                true
            },
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let mut ans = Self::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if feeder.starts_with("]]")
            || feeder.starts_with(")") {
                if ans.elements.len() == 0 {
                    return None;
                }

                return Some(ans);
            }

            if Self::eat_paren(feeder, &mut ans, core) 
            || Self::eat_compare_op(feeder, &mut ans, core)
            || Self::eat_file_check_option(feeder, &mut ans, core)
            || Self::eat_not_and_or(feeder, &mut ans) 
            || Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            break;
        }
        None
    }
}
