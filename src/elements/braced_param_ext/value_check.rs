//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::{Word, WordMode};
use super::{BracedParamExtension, ExecError, ParseError, Parameter, Subword};

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub symbol: String,
    pub alter: Option<Word>,
}

impl BracedParamExtension for ValueCheck {
    fn exec(&mut self, v: &Parameter, text: &str, core: &mut ShellCore)
        -> Result<String, ExecError> {
        let mut check_ok = match self.symbol.starts_with(":") {
            true  => !text.is_empty(),
            false => core.db.exist(&v.text),
        };

        if self.symbol.ends_with("+") {
            check_ok = !check_ok;
        }
        //println!("{:?}", if check_ok { "チェックOK" } else {"処理が必要"} );
        if check_ok {
            self.alter = None;
            return Ok(text.to_string());
        }

        match self.symbol.as_ref() {
            "?" | ":?" => self.show_error(&v.text, core),
            "=" | ":=" => self.set_value(v, core),
            _ => self.replace(core),
        }
    }

    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> {
        Box::new(self.clone())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn get_alter(&self) -> Vec<Box<dyn Subword>> {
        match &self.alter {
            Some(w) => w.subwords.to_vec(),
            None => vec![],
        }
    }
}

impl ValueCheck {
    fn to_string(&mut self, core: &mut ShellCore)
    -> Result<String, ExecError> {
        self.alter.clone().unwrap().eval_as_value(core)
    }

    fn replace(&mut self, core: &mut ShellCore)
    -> Result<String, ExecError> {
        let alt = self.alter.clone().unwrap();
        self.alter = Some(alt.tilde_and_dollar_expansion(core)?);
        Ok("".to_string())
    }

    fn set_value(&mut self, v: &Parameter, core: &mut ShellCore)
    -> Result<String, ExecError> {
        let value = self.to_string(core)?;
        core.db.set_param(&v.text, &value, None)?;
        self.alter = None;
        Ok(value)
    }

    fn show_error(&mut self, name: &str, core: &mut ShellCore)
    -> Result<String, ExecError> {
        let value = self.to_string(core)?;
        let msg = format!("{}: {}", &name, &value);
        Err(ExecError::Other(msg))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_check_symbol();
        if len == 0 { 
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        let w = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &w.text;
        ans.alter = Some(w);
        Ok(Some(ans))
    }
}
