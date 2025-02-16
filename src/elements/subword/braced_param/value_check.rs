//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, Feeder, ShellCore};
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.symbol.as_deref() {
            Some(":-")   => {
                self.set_alter_word(core)?;
                Ok(text.clone())
            },
            Some(":?") => self.colon_question(name, text, core),
            Some(":=") => self.colon_equal(name, core),
            Some("-")  => self.minus(name, text, core),
            Some(":+") => self.colon_plus(text, core),
            Some("+")  => self.plus(name, text, core),
            _          => exit::internal("no operation"),
        }
    }

    fn set_alter_word(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let v = self.alternative_value.clone().ok_or(ExecError::OperandExpected("".to_string()))?;
        self.alternative_value = Some(v.tilde_and_dollar_expansion(core)? );
        let value = v.eval_as_value(core)?;//.ok_or(ExecError::OperandExpected("".to_string()))?;
        Ok(value.clone())
    }

    fn minus(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match core.db.has_value(&name) {
            false => {self.set_alter_word(core)?;},
            true  => self.alternative_value = None,
        }
        Ok(text.clone())
            /*
        self.alternative_value = None;
        self.symbol = None;
        Ok(text.clone())
            */
    }

    fn plus(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match core.db.has_value(&name) && ! core.db.is_array(&name) {
            true  => {self.set_alter_word(core)?;},
            false => self.alternative_value = None,
        }
        Ok(text.clone())
    }

    fn colon_plus(&mut self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        match text.is_empty() {
            true  => self.alternative_value = None,
            false => {self.set_alter_word(core)?;},
        }
        Ok(text.clone())
    }

    fn colon_equal(&mut self, name: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        let value = self.set_alter_word(core)?;
        core.db.set_param(&name, &value, None)?;
        self.alternative_value = None;
        Ok(value)
    }

    fn colon_question(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        if core.db.has_value(&name) {
            self.alternative_value = None;
            return Ok(text.clone());
        }
        let value = self.set_alter_word(core)?;
        let msg = format!("{}: {}", &name, &value);
        Err(ExecError::Other(msg))
    }

    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return Ok(false);
        }

        let mut info = ValueCheck::default();

        let symbol = feeder.consume(num);
        info.symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        info.alternative_value = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core)?);

        ans.value_check = Some(info);
        Ok(true)
    }
}
