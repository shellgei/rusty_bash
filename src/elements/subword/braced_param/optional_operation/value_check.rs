//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, Feeder, ShellCore};
use crate::elements::subword::Subword;
use crate::elements::word::{Word, WordMode};
use crate::error::arith::ArithError;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::super::{Subscript, Variable};
use super::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub subscript: Option<Subscript>,
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl OptionalOperation for ValueCheck {
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, param: &Variable, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.set(&param.name, &param.index, text, core)
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
    fn is_value_check(&self) -> bool {true}

    fn get_alternative(&self) -> Vec<Box<dyn Subword>> {
        match &self.alternative_value {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }

    fn set_heredoc_flag(&mut self) { 
        self.alternative_value
            .iter_mut()
            .for_each(|e| e.set_heredoc_flag());
    }
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, sub: &Option<Subscript>, text: &String, core: &mut ShellCore) -> Result<String, ExecError> {
        self.subscript = sub.clone();

        let sym = self.symbol.clone().unwrap();

        let exist = match sym.starts_with(":") {
            true  => text != "",
            false => self.exist(name, core)?,
        };

        match sym.as_ref() {
            "?" | ":?" => self.colon_question(name, text, core),
            "=" | ":=" => self.colon_equal(name, text, core),
            "-" | ":-" => self.replace(text, core, !exist),
            "+" | ":+"  => self.replace(text, core, exist),
            _    => exit::internal("no operation"),
        }
    }

    fn exist(&mut self, name: &String, core: &mut ShellCore)
    -> Result<bool, ExecError> {
        if core.db.is_array(&name) {
            if core.db.get_vec(&name, false)?.is_empty() {
                return Ok(false);
            }
        }
        
        if let Some(sub) = self.subscript.as_mut() {
            if sub.eval(core, &name).is_ok() {
                if core.db.has_array_value(&name, &sub.text) {
                    return Ok(true);
                }
            }
        }

        Ok(core.db.has_value(name))
    }

    fn set_alter_word(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let v = match &self.alternative_value {
            Some(av) => av.clone(), 
            None => return Err(ArithError::OperandExpected("".to_string()).into()),
        };
        self.alternative_value = Some(v.tilde_and_dollar_expansion(core)? );
        let value = v.eval_as_value(core)?;
        Ok(value.clone())
    }

    fn replace(&mut self, text: &String, core: &mut ShellCore, exist: bool)
    -> Result<String, ExecError> { 
        match exist {
            true  => {self.set_alter_word(core)?;},
            false => self.alternative_value = None,
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
        let mode = WordMode::ParamOption(vec!["}".to_string()]);
        if let Some(w) = Word::parse(feeder, core, Some(mode))? {
            ans.text += &w.text.clone();
            ans.alternative_value = Some(w);
        }else{
            ans.alternative_value = Some(Word::default());
        }

        Ok(Some(ans))
    }
}
