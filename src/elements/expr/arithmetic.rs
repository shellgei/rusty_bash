//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;
pub mod elem;
mod parser;
mod rev_polish;

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::utils::exit;
use self::calculator::calculate;
use self::elem::ArithElem;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct ArithmeticExpr {
    pub text: String,
    pub elements: Vec<ArithElem>,
    output_base: String,
    hide_base: bool,
    in_ternary: bool,
}

impl ArithmeticExpr {
    pub fn eval_doller(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut txt = String::new();
        for e in &self.elements {
            match e {
                ArithElem::Word(w, inc) => {
                    if w.text.contains("'") {
                        return Err(ExecError::OperandExpected(w.text.clone()));
                    }
                    let text = w.eval_as_value(core)?;
                    let word = ArithElem::Word( Word::from(&text), *inc);
                    txt += &word.to_string();
                },
                e => txt += &e.to_string(),
            }
        }

        if let Some(a) = Self::parse_after_eval(&mut Feeder::new(&txt), core, "")? {
            self.text = a.text;
            self.elements = a.elements;
        }

        if core.db.flags.contains('x') {
            let ps4 = core.get_ps4();
            eprintln!("\r{} (( {} ))\r", ps4, &self.text);
        }

        Ok(())
    }

    pub fn eval(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut cp = self.clone();
        cp.eval_doller(core)?;

        match cp.eval_elems(core, true)? {
            ArithElem::Integer(n) => self.ans_to_string(n),
            ArithElem::Float(f)   => Ok(f.to_string()),
            e => return Err(ExecError::OperandExpected(e.to_string())),
        }
    }

    pub fn eval_as_assoc_index(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        self.eval_doller(core)?;
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

    pub fn eval_as_int(&mut self, core: &mut ShellCore) -> Option<i128> {
        let _ = self.eval_doller(core);

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
        }
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        calculate(&es, core)
    }

    fn ans_to_string(&self, n: i128) -> Result<String, ExecError> {
        let base_str = self.output_base.clone();

        if base_str == "10" {
            return Ok(n.to_string());
        }

        let base = match base_str.parse::<i128>() {
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

    fn dec_to_str(nums: &Vec<u8>, base: i128) -> String {
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
    }

    fn preinc_to_unarys(&mut self, ans: &mut Vec<ArithElem>, pos: usize, inc: i128) -> i128 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (Some(&ArithElem::Variable(_, _, _)), Some(&ArithElem::Word(_, _)))
                                     => ans.push(ArithElem::BinaryOp(pm.clone())),
            (_, None)
            | (_, Some(&ArithElem::Variable(_, _, _))) => return inc,
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
                ArithElem::Variable(_, _, _) => {
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
