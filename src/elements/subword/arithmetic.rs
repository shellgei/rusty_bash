//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::calc::Calc;
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct Arithmetic {
    pub text: String,
    pub calc: Calc,
}

impl Subword for Arithmetic {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        match self.calc.eval(core) {
            Some(s) => self.text = s,
            None    => return false,
        }
        true
    }
}

impl Arithmetic {
    fn new() -> Self {
        Self {
            text: String::new(),
            calc: Calc::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$((") {
            return None;
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(3);

        if let Some(c) = Calc::parse(feeder, core) {
            ans.text += &c.text;
            ans.text += &feeder.consume(2);
            ans.calc = c;
            feeder.pop_backup();
//            dbg!("{:?}", &ans);
            return Some(ans);
        }
    
        feeder.rewind();
        None
    }
}
