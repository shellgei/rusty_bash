//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;
mod elem;
mod parser;
mod rev_polish;
mod trenary;
mod word;
mod int;
mod float;

use crate::{error_message, ShellCore};
use self::calculator::calculate;
use self::elem::Elem;
use super::word::Word;

#[derive(Debug, Clone)]
pub struct ArithmeticExpr {
    pub text: String,
    elements: Vec<Elem>,
    output_base: String,
    hide_base: bool,
}

impl ArithmeticExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        match self.eval_elems(core) {
            Ok(Elem::Integer(n)) => self.ans_to_string(n),
            Ok(Elem::Float(f))   => Some(f.to_string()),
            Err(msg) => {
                eprintln!("sush: {}: {}", &self.text, msg);
                None
            },
            _ => error_message::internal("invalid calculation result"),
        }
    }

    pub fn eval_elems(&mut self, core: &mut ShellCore) -> Result<Elem, String> {
        if self.elements.len() == 0 {
            return Err("operand expexted (error token: \")\")".to_string());
        }
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        calculate(&es, core)
    }

    fn ans_to_string(&self, n: i64) -> Option<String> {
        let base_str = self.output_base.clone();

        if base_str == "10" {
            return Some(n.to_string());
        }

        let base = match base_str.parse::<i64>() {
            Ok(b) => b,
            _     => {
                eprintln!("sush: {0}: invalid arithmetic base (error_message token is \"{0}\")", base_str);
                return None;
            },
        };

        if base <= 1 || base > 64 {
            eprintln!("sush: {0}: invalid arithmetic base (error_message token is \"{0}\")", base_str);
            return None;
        }

        let mut tmp = n.abs();
        let mut digits = vec![];
        while tmp != 0 {
            digits.insert(0, (tmp % base) as u8);
            tmp /= base;
        }

        let mut ans = Self::dec_to_str(&digits, base);
        if ! self.hide_base {
            ans = base_str + "#" + &ans;
        }

        if n < 0 {
            ans.insert(0, '-');
        }

        Some(ans)
    }

    fn dec_to_str(nums: &Vec<u8>, base: i64) -> String {
        let shift = if base <= 0 {
            |n| n + '0' as u8
        }else if base <= 36 {
            |n| if n < 10 { n + '0' as u8 }
                else { n - 10 + 'A' as u8 } 
        }else{
            |n| if n < 10 { n + '0' as u8 }
                else if n < 36 { n - 10 + 'a' as u8 } 
                else if n < 62 { n - 36 + 'A' as u8 } 
                else if n == 62 { '@' as u8 } 
                else { '_' as u8 } 
        };

        let ascii = nums.iter().map(|n| shift(*n) ).collect::<Vec<u8>>();
        std::str::from_utf8(&ascii).unwrap().to_string()
    }

    fn eval_in_cond(&mut self, core: &mut ShellCore) -> Result<Elem, String> {
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        match calculate(&es, core) {
            Ok(ans)      => Ok(ans),
            Err(err_msg) => return Err(err_msg),
        }
    }

    fn preinc_to_unarys(&mut self, ans: &mut Vec<Elem>, pos: usize, inc: i64) -> i64 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (_, None) 
            | (_, Some(&Elem::Word(_, _))) => return inc,
            (Some(&Elem::Integer(_)), _)
            | (Some(&Elem::Float(_)), _)   => ans.push(Elem::BinaryOp(pm.clone())),
            _                              => ans.push(Elem::UnaryOp(pm.clone())),
        }
        ans.push(Elem::UnaryOp(pm));
        0
    }

    fn decompose_increments(&mut self) -> Result<Vec<Elem>, String> {
        let mut ans = vec![];
        let mut pre_increment = 0;

        let len = self.elements.len();
        for i in 0..len {
            let e = self.elements[i].clone();
            pre_increment = match e {
                Elem::Word(_, _) => {
                    if pre_increment != 0 {
                        ans.push(Elem::Increment(pre_increment));
                    }
                    ans.push(e);
                    0
                },
                Elem::Increment(n) => self.preinc_to_unarys(&mut ans, i, n),
                _ => {
                    ans.push(self.elements[i].clone());
                    0
                },
            };
        }

        match pre_increment {
            1  => Err(error_message::syntax("++")),
            -1 => Err(error_message::syntax("--")),
            _  => Ok(ans),
        }
    }

    pub fn new() -> ArithmeticExpr {
        ArithmeticExpr {
            text: String::new(),
            elements: vec![],
            output_base: "10".to_string(),
            hide_base: false,
        }
    }
}
