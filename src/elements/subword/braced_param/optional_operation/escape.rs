//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::Variable;
use super::OptionalOperation;
use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct Escape {
    pub text: String,
    pub symbol: String,
}

impl OptionalOperation for Escape {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(&mut self, _: &Variable, text: &str, _: &mut ShellCore) -> Result<String, ExecError> {
        self.replace_single_data(text)
    }

    fn init_array(
        &mut self,
        param: &Variable,
        array: &mut Vec<String>,
        _: &mut String,
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        if param.name == "@" || param.name == "*" {
            *array = core.db.get_position_params();
            for elem in array.iter_mut() {
                *elem = self.replace_single_data(elem)?;
            }
            return Ok(());
        }

        if core.db.is_assoc(&param.name) {
            array.clear();
            for key in core.db.get_indexes_all(&param.name) {
                let value = core.db.get_elem(&param.name, &key).unwrap_or_default();
                array.push(self.replace_array_elem(&key, &value)?);
            }
            return Ok(());
        }

        *array = core.db.get_vec(&param.name, true)?;
        for (i, elem) in array.iter_mut().enumerate() {
            *elem = self.replace_array_elem(&i.to_string(), elem)?;
        }

        Ok(())
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {
        Box::new(self.clone())
    }
    fn has_array_replace(&self) -> bool {
        true
    }
}

impl Escape {
    pub fn replace_single_data(&self, text: &str) -> Result<String, ExecError> {
        match self.symbol.as_ref() {
            "k" | "K" | "Q" => {
                let text = format!("'{text}'");
                return Ok(text);
            }
            _ => {}
        }
        Ok(text.to_string())
    }

    pub fn replace_array_elem(&self, pos: &str, text: &str) -> Result<String, ExecError> {
        match self.symbol.as_ref() {
            "k" => {
                let text = format!("{pos} {text}");
                return Ok(text);
            }
            "K" => {
                let text = format!("{pos} \"{text}\"");
                return Ok(text);
            }
            "Q" => {
                let text = format!("'{text}'");
                return Ok(text);
            }
            _ => {}
        }
        Ok(text.to_string())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if !feeder.starts_with("@") {
            return None;
        }

        let mut ans = Escape::default();
        if feeder.scanner_escape_directive_in_braced_param(core) == 2 {
            ans.text = feeder.consume(1);
            ans.symbol = feeder.consume(1);
            ans.text += &ans.symbol;
        } else {
            return None;
        }

        Some(ans)
    }
}
