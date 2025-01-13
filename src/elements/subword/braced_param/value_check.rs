//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, String> {
        match self.symbol.as_deref() {
            Some(":-")   => {
                self.set_alter_word(core)?;
                Ok(text.clone())
            },
            Some(":?") => self.colon_question(name, text, core),
            Some(":=") => self.colon_equal(name, core),
            Some("-")  => self.minus(text),
            Some(":+") => self.colon_plus(text, core),
            Some("+")  => self.plus(name, text, core),
            _          => Err("no operation".to_string()),
        }
    }

    fn set_alter_word(&mut self, core: &mut ShellCore) -> Result<String, String> {
        let v = self.alternative_value.clone().ok_or("no alternative value")?;
        self.alternative_value = Some(v.tilde_and_dollar_expansion(core)? );
        let value = v.eval_as_value(core).ok_or("parse error")?;
        Ok(value.clone())
    }

    fn minus(&mut self, text: &String) -> Result<String, String> {
        self.alternative_value = None;
        self.symbol = None;
        Ok(text.clone())
    }

    fn plus(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, String> {
        match core.db.has_value(&name) {
            true  => {self.set_alter_word(core)?;},
            false => self.alternative_value = None,
        }
        Ok(text.clone())
    }

    fn colon_plus(&mut self, text: &String, core: &mut ShellCore) -> Result<String, String> {
        match text.is_empty() {
            true  => self.alternative_value = None,
            false => {self.set_alter_word(core)?;},
        }
        Ok(text.clone())
    }

    fn colon_equal(&mut self, name: &String, core: &mut ShellCore) -> Result<String, String> {
        let value = self.set_alter_word(core)?;
        core.db.set_param(&name, &value, None)?;
        self.alternative_value = None;
        Ok(value)
    }

    fn colon_question(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, String> {
        if core.db.has_value(&name) {
            self.alternative_value = None;
            return Ok(text.clone());
        }
        let value = self.set_alter_word(core)?;
        let msg = format!("{}: {}", &name, &value);
        Err(msg)
    }

    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return false;
        }

        let mut info = ValueCheck::default();

        let symbol = feeder.consume(num);
        info.symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        info.alternative_value = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core));

        ans.value_check = Some(info);
        true
    }
}
