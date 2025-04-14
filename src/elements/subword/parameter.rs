//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::splitter;
use crate::error::exec::ExecError;
use crate::elements::subword::simple::SimpleSubword;
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct Parameter {
    pub text: String,
    pub array: Option<Vec<String>>,
}

impl Subword for Parameter {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        if ! self.text.starts_with("$") {
            return Ok(vec![]);
        }

        if self.text == "$*" || self.text == "$@" {
            self.array = Some(core.db.get_position_params());
        }

        let value = core.db.get_param(&self.text[1..]).unwrap_or(String::new());
        self.text = value.to_string();
        Ok(vec![])
    }

    fn is_array(&self) -> bool {self.text == "$@"}

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ 
        if ifs.starts_with(" ") || self.array.is_none() {
            let f = |s| Box::new( SimpleSubword {text: s}) as Box<dyn Subword>;

            let ans = splitter::split(&self.get_text(), ifs, prev_char);
            return ans.iter().map(|s| (f(s.0.to_string()), s.1)).collect();
        }

        let mut ans = vec![];
        let mut tmp = SimpleSubword{ text: "".to_string() };
        for p in self.array.clone().unwrap() {
            tmp.text = p.clone();
            ans.push((tmp.boxed_clone(), true));
        }
        ans
    }
}

impl Parameter {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_dollar_special_and_positional_param(core) {
            0 => None,
            n => {
                let mut ans = Self::default();
                ans.text = feeder.consume(n);
                Some(ans)
            },
        }
    }
}
