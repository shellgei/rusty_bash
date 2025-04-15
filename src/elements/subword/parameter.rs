//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::splitter;
use crate::error::exec::ExecError;
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct Parameter {
    pub text: String,
    pub array: Option<Vec<String>>,
}

impl Subword for Parameter {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if ! self.text.starts_with("$") {
            return Ok(());
        }

        if self.text == "$*" || self.text == "$@" {
            self.array = Some(core.db.get_position_params());
        }

        self.text = core.db.get_param(&self.text[1..]).unwrap_or(String::new());
        Ok(())
    }

    fn is_array(&self) -> bool {self.text == "$@"}

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ 
        if ifs.starts_with(" ") || self.array.is_none() {
            return splitter::split(&self.get_text(), ifs, prev_char).iter()
                .map(|s| ( From::from(&s.0), s.1)).collect();
        }

        let mut ans = vec![];
        for p in self.array.clone().unwrap() {
            ans.push( (From::from(&p), true) );
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
