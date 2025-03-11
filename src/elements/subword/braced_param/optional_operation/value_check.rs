//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, Feeder, ShellCore};
use crate::elements::subword::{Subword, BracedParam};
use crate::elements::subword::braced_param::Word;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::super::{Subscript, Param};
use super::OptionalOperation;

impl OptionalOperation for ValueCheck {
    fn exec(&mut self, param: &Param, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.set(&param.name, &param.subscript, text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
    fn is_value_check(&self) -> bool {true}

    fn get_alternative(&self) -> Vec<Box<dyn Subword>> {
        match &self.alternative_value {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub subscript: Option<Subscript>,
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, sub: &Option<Subscript>, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.subscript = sub.clone();
        match self.symbol.as_deref() {
            Some(":-")   => self.colon_minus(text, core),
            Some(":?") => self.colon_question(name, text, core),
            Some(":=") => self.colon_equal(name, text, core),
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
    }

    fn plus(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> { 
        if core.db.is_array(&name) {
            if core.db.get_array_all(&name).is_empty() {
                self.alternative_value = None;
                return Ok(text.clone());
            }
        }
        
        if let Some(sub) = self.subscript.as_mut() {
            if sub.eval(core, &name).is_ok() {
                if core.db.has_array_value(&name, &sub.text) {
                    self.set_alter_word(core)?;
                    return Ok(text.clone());
                }
            }
        }

        match core.db.has_value(&name) {
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

    fn colon_equal(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        if text != "" {
            self.alternative_value = None;
            return Ok(text.clone());
        }

        let value = self.set_alter_word(core)?;
        core.db.set_param(&name, &value, None)?;
        self.alternative_value = None;
        Ok(value)
    }

    fn colon_minus(&mut self, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        if text != "" {
            self.alternative_value = None;
            return Ok(text.clone());
        }

        self.set_alter_word(core)?;
        Ok(text.clone())
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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return Ok(None);
        }

        let mut ans = ValueCheck::default();

        let symbol = feeder.consume(num);
        ans.symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        ans.alternative_value = Some(BracedParam::eat_subwords(feeder, vec!["}"], core)?);

        Ok(Some(ans))
    }
}
