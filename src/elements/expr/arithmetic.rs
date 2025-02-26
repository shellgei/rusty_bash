//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod array_elem;
mod calculator;
pub mod elem;
mod parser;
mod rev_polish;
mod trenary;
pub mod word;
mod int;
mod float;

use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::utils::exit;
use self::calculator::calculate;
use self::elem::ArithElem;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct ArithmeticExpr {
    pub text: String,
    elements: Vec<ArithElem>,
    output_base: String,
    hide_base: bool,
}

impl ArithmeticExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.eval_elems(core, true)? {
            ArithElem::Integer(n) => self.ans_to_string(n),
            ArithElem::Float(f)   => Ok(f.to_string()),
            _ => exit::internal("invalid calculation result"),
        }
    }

    pub fn eval_as_assoc_index(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut ans = String::new();

        for e in &self.elements {
            match e {
                ArithElem::Word(w, i) => {
                    match w.eval_as_value(core) {
                        Ok(s) => {
                            ans += &s;
                            if *i > 0 {
                                ans += "++";
                            }else if *i < 0 {
                                ans += "--";
                            }
                        },
                        Err(e) => return Err(e),
                    }
                },
                _ => ans += &e.to_string(),
            }
        }

        Ok(ans)
    }

    pub fn eval_as_int(&mut self, core: &mut ShellCore) -> Option<i64> {
        match self.eval_elems(core, true) {
            Ok(ArithElem::Integer(n)) => Some(n),
            Ok(ArithElem::Float(f))   => {
                eprintln!("sush: {}: Not integer. {}", &self.text, f);
                None
            },
            Err(msg) => {
                eprintln!("sush: {}: {:?}", &self.text, msg);
                None
            },
            _ => exit::internal("invalid calculation result"),
        }
    }

    pub fn eval_elems(&mut self, core: &mut ShellCore, permit_empty: bool) -> Result<ArithElem, ExecError> {
        if self.elements.is_empty() && ! permit_empty {
            return Err(ExecError::OperandExpected("\")\"".to_string()));
            //return Err("operand expexted (error token: \")\")".to_string());
        }
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        calculate(&es, core)
    }

    fn ans_to_string(&self, n: i64) -> Result<String, ExecError> {
        let base_str = self.output_base.clone();

        if base_str == "10" {
            return Ok(n.to_string());
        }

        let base = match base_str.parse::<i64>() {
            Ok(b) => b,
            _     => {
                return Err(ExecError::InvalidBase(base_str));
            },
        };

        if base <= 1 || base > 64 {
            return Err(ExecError::InvalidBase(base_str));
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

        Ok(ans)
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

    fn eval_in_cond(&mut self, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
        let es = self.decompose_increments()?;

        calculate(&es, core)
            /*
        match calculate(&es, core) {
            Ok(ans)      => Ok(ans),
            Err(err_msg) => return Err(err_msg),
        }
            */
    }

    fn preinc_to_unarys(&mut self, ans: &mut Vec<ArithElem>, pos: usize, inc: i64) -> i64 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (_, None)
            | (_, Some(&ArithElem::Word(_, _)))
            | (_, Some(&ArithElem::ArrayElem(_, _, _))) => return inc,
            (Some(&ArithElem::Integer(_)), _)
            | (Some(&ArithElem::Float(_)), _)   => ans.push(ArithElem::BinaryOp(pm.clone())),
            _                              => ans.push(ArithElem::UnaryOp(pm.clone())),
        }
        ans.push(ArithElem::UnaryOp(pm));
        0
    }

    fn decompose_increments(&mut self) -> Result<Vec<ArithElem>, ExecError> {
        let mut ans = vec![];
        let mut pre_increment = 0;

        let len = self.elements.len();
        for i in 0..len {
            let e = self.elements[i].clone();
            pre_increment = match e {
                ArithElem::Word(_, _) |
                ArithElem::ArrayElem(_, _, _) => {
                    if pre_increment != 0 {
                        ans.push(ArithElem::Increment(pre_increment));
                    }
                    ans.push(e);
                    0
                },
                ArithElem::Increment(n) => self.preinc_to_unarys(&mut ans, i, n),
                _ => {
                    ans.push(self.elements[i].clone());
                    0
                },
            };
        }

        match pre_increment {
            1  => Err(ExecError::OperandExpected("++".to_string())),
            -1 => Err(ExecError::OperandExpected("--".to_string())),
            _  => Ok(ans),
        }
    }

    pub fn new() -> ArithmeticExpr {
        ArithmeticExpr {
            output_base: "10".to_string(),
            ..Default::default()
        }
    }
}
