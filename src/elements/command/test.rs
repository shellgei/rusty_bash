//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore, Feeder};
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
enum Elem {
    FileCheckOption(String),
    Word(Word),
    Operand(String),
    RightParen,
    LeftParen,
    Not, // !
    And,  // &&
    Or,  // ||
    Ans(bool),
}

fn op_order(op: &Elem) -> u8 {
    match op {
        Elem::FileCheckOption(_) => 14,
        Elem::Not => 13,
        Elem::And | Elem::Or => 12,
        _ => 0,
    }
}

fn to_string(op: &Elem) -> String {
    match op {
        Elem::FileCheckOption(op) => op.to_string(),
        Elem::Word(w) => w.text.clone(),
        Elem::Operand(op) => op.to_string(),
        Elem::LeftParen => "(".to_string(),
        Elem::RightParen => ")".to_string(),
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
    let n = match stack.pop() {
        Some(Elem::Word(w)) => {
            match to_operand(&w, core) {
                Ok(op) => op,
                Err(e) => return Err(e),
            }
        },
        Some(elem) => elem,
        None       => return Err("no operand".to_string()),
    };
    Ok(n)
}

#[derive(Debug, Clone)]
pub struct TestCommand {
    text: String,
    elements: Vec<Elem>,
    paren_stack: Vec<char>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        let rev_pol = match self.rev_polish() {
            Ok(ans) => ans,
            _ => {
                core.data.set_param("?", "2");
                return;
            },
        };

        let mut stack = vec![];

        for e in rev_pol {
            let result = match e { 
                Elem::Word(_) => {
                    stack.push(e.clone());
                    Ok(())
                },
                Elem::FileCheckOption(ref op)  => {
                    Self::unary_operation(&op, &mut stack, core)
                },
                Elem::Not => match stack.pop() {
                    Some(Elem::Ans(res)) => {
                        stack.push(Elem::Ans(!res));
                        Ok(())
                    },
                    _ => Err("no operand to negate".to_string()),
                },
                _ => Err( error_message::syntax("TODO")),
            };
    
            if let Err(err_msg) = result {
                error_message::print(&err_msg, core, true);
                core.data.set_param("?", "2");
                return;
            }
        }
        if stack.len() != 1 { 

            if stack.len() > 1 {
                let err = error_message::syntax_in_cond_expr(&to_string(&stack[0]));
                error_message::print(&err, core, true);
                let err = format!("syntax error near `{}'", to_string(&stack[0]));
                error_message::print(&err, core, true);
            }
            core.data.set_param("?", "2");
            return;
        }   
    
        match stack.pop() {
            Some(Elem::Ans(true))  => core.data.set_param("?", "0"),
            Some(Elem::Ans(false)) => core.data.set_param("?", "1"),
            _  => {
                eprintln!("unknown syntax error_message");
                core.data.set_param("?", "2");
            },
        }  
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    fn rev_polish(&mut self) -> Result<Vec<Elem>, Elem> {
        let mut ans = vec![];
        let mut stack = vec![];
        let mut last = None;
    
        for e in &self.elements {
            let ok = match e {
                Elem::Word(_)    => {ans.push(e.clone()); true},
                Elem::LeftParen  => {stack.push(e.clone()); true},
                Elem::RightParen => Self::rev_polish_paren(&mut stack, &mut ans),
                op               => Self::rev_polish_op(&op, &mut stack, &mut ans),
            };
    
            if !ok {
                return Err(e.clone());
            }
    
            match (last, e) {
                ( Some(Elem::LeftParen), Elem::RightParen ) => return Err(e.clone()),
                _ => {},
            }
    
            last = Some(e.clone());
        }
    
        while stack.len() > 0 {
            ans.push(stack.pop().unwrap());
        }
    
        Ok(ans)
    }

    fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
        let operand = match pop_operand(stack, core) {
            Ok(v)  => v, 
            Err(e) => return Err(e + " to conditional unary operator"),
        };
        
        match operand {
            Elem::Operand(s) => Self::unary_calc(op, &s, stack),
            _ => error_message::internal("unknown operand"), 
        }
    }

    fn unary_calc(op: &str, s: &String, stack: &mut Vec<Elem>) -> Result<(), String> {
        match op {
            "-a"  => {
                let ans = Path::new(s).is_file();
                stack.push( Elem::Ans(ans) );
            },
            "-b"  => {
                let meta = match fs::metadata(s) {
                    Ok(m) => m,
                    _  => {
                        stack.push( Elem::Ans(false) );
                        return Ok(());
                    },
                };
                let ans = meta.file_type().is_block_device();
                stack.push( Elem::Ans(ans) );
            },
            _  => stack.push( Elem::Ans(false) ),
        }   
        Ok(())
    }

    fn rev_polish_paren(stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
        loop {
            match stack.last() {
                None => return false,
                Some(Elem::LeftParen) => {
                    stack.pop();
                    return true;
                },
                Some(_) => ans.push(stack.pop().unwrap()),
            }
        }
    }
    
    fn rev_polish_op(elem: &Elem,
                     stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
        loop {
            match stack.last() {
                None | Some(Elem::LeftParen) => {
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

    fn new() -> TestCommand {
        TestCommand {
            text: String::new(),
            elements: vec![],
            paren_stack: vec![],
            redirects: vec![],
            force_fork: false,
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

    fn eat_file_check_option(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_file_check_option(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(Elem::FileCheckOption(opt));

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

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if let Some(e) = ans.elements.last() {
            match e {
                Elem::FileCheckOption(_) => {
                    return false
                },
                _ => {},
            }
        }

        if feeder.starts_with("(") {
            ans.paren_stack.push( '(' );
            ans.elements.push( Elem::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.paren_stack.pop();
                ans.elements.push( Elem::RightParen );
                ans.text += &feeder.consume(1);
                return true;
            }
        }

        false
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
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        loop {
            if ! Self::eat_blank(feeder, &mut ans, core) {
                return None;
            }
            if feeder.starts_with("]]") {
                if ans.elements.len() == 0 {
                    return None;
                }

                ans.text += &feeder.consume(2);
                command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
                return Some(ans);
            }

            if Self::eat_file_check_option(feeder, &mut ans, core)
            || Self::eat_not_and_or(feeder, &mut ans) 
            || Self::eat_paren(feeder, &mut ans) 
            || Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            if feeder.starts_with("(") {
                let err = error_message::syntax_in_cond_expr("(");
                error_message::print(&err, core, true);
            }else {
                let err = error_message::syntax_in_cond_expr(")");
                error_message::print(&err, core, true);
            }

            break;
        }
    
        None
    }
}
