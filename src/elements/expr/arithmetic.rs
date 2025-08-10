//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;
pub mod elem;
mod parser;
mod rev_polish;

use self::calculator::calculate;
use self::elem::ArithElem;
use crate::elements::word::Word;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::utils::exit;
use crate::{Feeder, ShellCore};

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
                        let err = ArithError::OperandExpected(w.text.clone());
                        let arith_txt = self.text.trim_start().to_string();
                        return Err(ExecError::ArithError(arith_txt, err));
                    }
                    let text = w.eval_as_value(core)?;
                    let word = ArithElem::Word(Word::from(text.as_str()), *inc);
                    txt += &word.to_string();
                }
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

        let ans = match cp.eval_elems(core, true) {
            Ok(a) => a,
            Err(ExecError::ArithError(mut s, a)) => {
                if s.is_empty() {
                    s = cp.text.trim_start().to_string();
                }
                return Err(ExecError::ArithError(s, a));
            }
            Err(e) => return Err(e),
        };

        match ans {
            ArithElem::Integer(n) => match self.ans_to_string(n) {
                Ok(ans) => Ok(ans),
                Err(a) => Err(ExecError::ArithError(cp.text, a)),
            },
            ArithElem::Float(f) => Ok(f.to_string()),
            e => Err(ExecError::ArithError(
                cp.text,
                ArithError::OperandExpected(e.to_string()),
            )),
        }
    }

    pub fn eval_as_int(&mut self, core: &mut ShellCore) -> Result<i128, ExecError> {
        let _ = self.eval_doller(core);

        match self.eval_elems(core, true)? {
            ArithElem::Integer(n) => Ok(n),
            ArithElem::Float(f) => {
                let msg = format!("sush: {}: Not integer. {}", &self.text, f);
                Err(ExecError::Other(msg))
            }
            _ => exit::internal("invalid calculation result"),
        }
    }

    pub fn eval_elems(
        &mut self,
        core: &mut ShellCore,
        permit_empty: bool,
    ) -> Result<ArithElem, ExecError> {
        if self.elements.is_empty() && !permit_empty {
            return Err(ArithError::OperandExpected("\")\"".to_string()).into());
        }
        let es = self.decompose_increments()?;

        calculate(&es, core)
    }

    fn ans_to_string(&self, n: i128) -> Result<String, ArithError> {
        let base_str = self.output_base.clone();

        if base_str == "10" {
            return Ok(n.to_string());
        }

        let base = match base_str.parse::<i128>() {
            Ok(b) => b,
            _ => {
                return Err(ArithError::InvalidBase(base_str));
            }
        };

        if base <= 1 || base > 64 {
            return Err(ArithError::InvalidBase(base_str));
        }

        let mut tmp = n.abs();
        let mut digits = vec![];
        while tmp != 0 {
            digits.insert(0, (tmp % base) as u8);
            tmp /= base;
        }

        let mut ans = Self::dec_to_str(&digits, base);
        if !self.hide_base {
            ans = base_str + "#" + &ans;
        }

        if n < 0 {
            ans.insert(0, '-');
        }

        Ok(ans)
    }

    fn dec_to_str(nums: &[u8], base: i128) -> String {
        let shift = if base <= 0 {
            |n| n + b'0'
        } else if base <= 36 {
            |n| {
                if n < 10 {
                    n + b'0'
                } else {
                    n - 10 + b'A'
                }
            }
        } else {
            |n| {
                if n < 10 {
                    n + b'0'
                } else if n < 36 {
                    n - 10 + b'a'
                } else if n < 62 {
                    n - 36 + b'A'
                } else if n == 62 {
                    b'@'
                } else {
                    b'_'
                }
            }
        };

        let ascii = nums.iter().map(|n| shift(*n)).collect::<Vec<u8>>();
        std::str::from_utf8(&ascii).unwrap().to_string()
    }

    fn eval_in_cond(&mut self, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
        let es = self.decompose_increments()?;
        calculate(&es, core)
    }

    fn preinc_to_unarys(&mut self, ans: &mut Vec<ArithElem>, pos: usize, inc: i128) -> i128 {
        let pm = match inc {
            1 => "+",
            -1 => "-",
            _ => return 0,
        }
        .to_string();

        match (&ans.last(), &self.elements.get(pos + 1)) {
            (Some(&ArithElem::Variable(_, _, _)), Some(&ArithElem::Word(_, _))) => {
                ans.push(ArithElem::BinaryOp(pm.clone()))
            }
            (_, None) | (_, Some(&ArithElem::Variable(_, _, _))) => return inc,
            (Some(&ArithElem::Integer(_)), _) | (Some(&ArithElem::Float(_)), _) => {
                ans.push(ArithElem::BinaryOp(pm.clone()))
            }
            _ => ans.push(ArithElem::UnaryOp(pm.clone())),
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
                }
                ArithElem::Increment(n) => self.preinc_to_unarys(&mut ans, i, n),
                _ => {
                    ans.push(self.elements[i].clone());
                    0
                }
            };
        }

        match pre_increment {
            //↓treated as + or - in error messages
            1 => Err(ArithError::OperandExpected("+".to_string()).into()),
            -1 => Err(ArithError::OperandExpected("-".to_string()).into()),
            _ => Ok(ans),
        }
    }

    pub fn new() -> ArithmeticExpr {
        ArithmeticExpr {
            output_base: "10".to_string(),
            ..Default::default()
        }
    }
}
