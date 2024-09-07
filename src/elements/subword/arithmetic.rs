//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::arithmetic_expression::ArithmeticExpr;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Arithmetic {
    pub text: String,
    pub arith: Vec<ArithmeticExpr>,
}

impl Subword for Arithmetic {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        for a in &mut self.arith {
            match a.eval(core) {
                Some(s) => self.text = s,
                None    => return false,
            }
        }

        true
    }
}

impl Arithmetic {
    fn new() -> Self {
        Self {
            text: String::new(),
            arith: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$((") {
            return None;
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(3);

        loop {
            if let Some(c) = ArithmeticExpr::parse(feeder, core) {
                if feeder.starts_with(",") {
                    ans.text += &c.text;
                    ans.text += &feeder.consume(1);
                    ans.arith.push(c);
                    continue;
                }

                if feeder.starts_with("))") {
                    ans.text += &c.text;
                    ans.text += &feeder.consume(2);
                    ans.arith.push(c);
                    feeder.pop_backup();
                    return Some(ans);
                }
    
                break;
            }else{
                break;
            }
        }

        feeder.rewind();
        None
    }
}
