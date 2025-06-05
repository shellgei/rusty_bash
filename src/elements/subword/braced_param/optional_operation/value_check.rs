//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::Subword;
use crate::elements::word::{Word, WordMode};
use crate::error::arith::ArithError;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use super::super::Variable;
use super::OptionalOperation;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl OptionalOperation for ValueCheck {
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, param: &Variable, text: &String, core: &mut ShellCore)
    -> Result<String, ExecError> {
        let sym = self.symbol.clone().unwrap();
        let mut check_ok = match sym.starts_with(":") {
            true  => text != "",
            false => param.exist(core)?,
        };

        if sym.ends_with("+") {
            check_ok = !check_ok;
        }

        if check_ok {
            self.alternative_value = None;
            return Ok(text.clone());
        }

        match sym.as_ref() {
            "?" | ":?" => self.show_error(&param.name, core),
            "=" | ":=" => self.set_value(&param.name, core),
            _  => self.replace(text, core),
        }
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
    fn set_alter_word(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let v = match &self.alternative_value {
            Some(av) => av.clone(), 
            None => return Err(ArithError::OperandExpected("".to_string()).into()),
        };
        self.alternative_value = Some(v.tilde_and_dollar_expansion(core)? );
        Ok(v.eval_as_value(core)?)
    }

    fn replace(&mut self, text: &String, core: &mut ShellCore)
    -> Result<String, ExecError> { 
        self.set_alter_word(core)?;
        Ok(text.clone())
    }

    fn set_value(&mut self, name: &String, core: &mut ShellCore)
    -> Result<String, ExecError> {
        let value = self.set_alter_word(core)?;
        core.db.set_param(&name, &value, None)?;
        self.alternative_value = None;
        Ok(value)
    }

    fn show_error(&mut self, name: &String, core: &mut ShellCore)
    -> Result<String, ExecError> {
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
        ans.alternative_value = Some(Word::default());

        if let Some(w) = Word::parse(feeder, core, Some(mode))? {
            ans.text += &w.text.clone();
            ans.alternative_value = Some(w);
        }

        Ok(Some(ans))
    }
}
