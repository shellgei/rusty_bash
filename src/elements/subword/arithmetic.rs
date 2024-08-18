//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::arithmetic_expression::ArithmeticExpr;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Arithmetic {
    pub text: String,
    pub arith: ArithmeticExpr,
}

impl Subword for Arithmetic {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        let backup = core.data.get_param("_");
        core.data.set_param("_", ""); //_ is used for setting base of number output
        
        let result = match self.arith.eval(core) {
            Some(s) => {self.text = s; true},
            None    => {false},
        };

        core.data.set_param("_", &backup);
        result
    }
}

impl Arithmetic {
    fn new() -> Self {
        Self {
            text: String::new(),
            arith: ArithmeticExpr::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$((") {
            return None;
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(3);

        if let Some(c) = ArithmeticExpr::parse(feeder, core) {
            if ! feeder.starts_with("))") {
                feeder.rewind();
                return None;
            }

            ans.text += &c.text;
            ans.text += &feeder.consume(2);
            ans.arith = c;
            feeder.pop_backup();
            return Some(ans);
        }
    
        feeder.rewind();
        None
    }
}
