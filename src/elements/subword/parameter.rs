//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Subword;
//use crate::elements::parameter::Parameter;
use super::splitter;
use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Parameter {
    pub text: String,
    pub array: Option<Vec<String>>,
}

impl Subword for Parameter {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if !self.text.starts_with("$") {
            return Ok(());
        }

        if self.text == "$*" || self.text == "$@" {
            self.array = Some(core.db.get_position_params());
        }

        self.text = core.db.get_param(&self.text[1..])?;
        Ok(())
    }

    fn is_array(&mut self, _: &mut ShellCore) -> bool {
        self.text == "$@"
    }

    fn split(&self, ifs: &str, strip_left: bool) -> Option<Vec<(Box<dyn Subword>, bool)>> {
        if self.text.is_empty() {
            return None;
        }

        if ifs.contains(" ") || self.array.is_none() {
            //TODO: add \t and \n ?
            return splitter::split(self.get_text(), ifs, strip_left);
        }

        let mut ans = vec![];
        for p in self.array.clone().unwrap() {
            ans.push((From::from(&p), true));
        }
        Some(ans)
    }

    fn is_simple_param(&self) -> bool {
        true
    }
}

impl Parameter {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        match feeder.scanner_dollar_special_and_positional_param(core) {
            0 => None,
            n => {
                let ans = Self {
                    text: feeder.consume(n),
                    ..Default::default()
                };
                Some(ans)
            }
        }
    }
}
