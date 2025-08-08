//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::Variable;
use super::OptionalOperation;
use crate::elements::subword::SingleQuoted;
use crate::elements::subword::Subword;
use crate::elements::word::{Word, WordMode};
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub text: String,
    pub symbol: Option<String>,
    alternative_value: Option<Word>,
    in_double_quoted: bool,
}

impl OptionalOperation for ValueCheck {
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn exec(
        &mut self,
        variable: &Variable,
        text: &String,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        let sym = self.symbol.clone().unwrap();
        let mut check_ok = match sym.starts_with(":") {
            true => text != "",
            false => variable.exist(core)?,
        };

        if sym.ends_with("+") {
            check_ok = !check_ok;
        }

        if check_ok {
            self.alternative_value = None;
            return Ok(text.clone());
        }

        match sym.as_ref() {
            "?" | ":?" => self.show_error(&variable.name, core),
            "=" | ":=" => self.set_value(&variable, core),
            _ => self.replace(core),
        }
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {
        Box::new(self.clone())
    }
    fn is_value_check(&self) -> bool {
        true
    }

    fn get_alternative(&self) -> Vec<Box<dyn Subword>> {
        match &self.alternative_value {
            Some(w) => w.subwords.to_vec(),
            None => vec![],
        }
    }

    fn set_heredoc_flag(&mut self) {
        self.alternative_value
            .iter_mut()
            .for_each(|e| e.set_heredoc_flag());
    }

    fn array_to_single(&mut self) -> bool {
        self.alternative_value.is_some()
    }
}

impl ValueCheck {
    fn set_alter_word(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let v = match &self.alternative_value {
            Some(av) => av.clone(),
            None => return Err(ArithError::OperandExpected("".to_string()).into()),
        };

        self.alternative_value = Some(v.tilde_and_dollar_expansion(core)?);
        if self.in_double_quoted {
            for sw in self.alternative_value.as_mut().unwrap().subwords.iter_mut() {
                if sw.get_text().starts_with("'") {
                    Self::apply_single_quote_rule(sw);
                }
            }
        }
        Ok(v.eval_as_value(core)?)
    }

    fn apply_single_quote_rule(sw: &mut Box<dyn Subword>) {
        let mut escaped = false;
        let mut ans = String::new();
        for c in sw.get_text().chars() {
            if escaped || c == '\\' {
                escaped = !escaped;
                if c == '"' {
                    ans.pop();
                }
            } else if c == '"' {
                continue;
            }
            ans.push(c);
        }

        ans.insert(0, '\'');
        ans.push('\'');
        *sw = Box::new(SingleQuoted {
            text: ans.to_string(),
        });
    }

    fn replace(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        self.set_alter_word(core)
    }

    fn set_value(
        &mut self,
        variable: &Variable,
        core: &mut ShellCore,
    ) -> Result<String, ExecError> {
        let mut value = self.set_alter_word(core)?;

        if core.db.is_int(&variable.name) {
            value = utils::string_to_calculated_string(&value, core)?;
        }
        if core.db.has_flag(&variable.name, 'l') {
            value = value.to_lowercase();
        } else if core.db.has_flag(&variable.name, 'u') {
            value = value.to_uppercase();
        }

        variable.clone().set_value(&value, core)?;
        self.alternative_value = None;
        Ok(value)
    }

    fn show_error(&mut self, name: &String, core: &mut ShellCore) -> Result<String, ExecError> {
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

        if feeder.nest.iter().any(|e| e.0 == "\"") {
            ans.in_double_quoted = true;
        }

        Ok(Some(ans))
    }
}
