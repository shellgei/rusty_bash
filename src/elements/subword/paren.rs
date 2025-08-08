//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{BracedParam, EscapedChar, Parameter, Subword, VarName};
use crate::elements::subword::{Arithmetic, CommandSubstitution, DoubleQuoted};
use crate::elements::word::{Word, WordMode};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct EvalLetParen {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for EvalLetParen {
    fn get_text(&self) -> &str {
        &self.text.as_ref()
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        self.connect_array(core)?;

        let word = Word::from(self.subwords.clone());
        self.text = word.eval_as_value(core)?;
        Ok(())
    }

    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)> {
        vec![]
    }
}

impl EvalLetParen {
    fn connect_array(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        for sw in self.subwords.iter_mut() {
            if sw.get_text() == "$*" || sw.get_text() == "${*}" {
                let params = core.db.get_position_params();
                let joint = core.db.get_ifs_head();
                *sw = From::from(&params.join(&joint));
            }
        }
        Ok(())
    }

    fn eat_element(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        let sw: Box<dyn Subword> = if let Some(a) = BracedParam::parse(feeder, core)? {
            Box::new(a)
        } else if let Some(a) = DoubleQuoted::parse(feeder, core, &None)? {
            Box::new(a)
        } else if let Some(a) = Arithmetic::parse(feeder, core)? {
            Box::new(a)
        } else if let Some(a) = CommandSubstitution::parse(feeder, core)? {
            Box::new(a)
        } else if let Some(a) = Parameter::parse(feeder, core) {
            Box::new(a)
        } else if let Some(a) = EscapedChar::parse(feeder, core) {
            Box::new(a)
        } else if let Some(a) = Self::parse_name(feeder, core) {
            Box::new(a)
        } else {
            return Ok(false);
        };

        ans.text += sw.get_text();
        ans.subwords.push(sw);
        Ok(true)
    }
    /*
    fn parse_escaped_char(feeder: &mut Feeder) -> Option<EscapedChar> {
        if feeder.starts_with("\\$") || feeder.starts_with("\\\\")
        || feeder.starts_with("\\\"") || feeder.starts_with("\\`") {
            return Some(EscapedChar{ text: feeder.consume(2) });
        }
        None
    }*/

    fn parse_name(feeder: &mut Feeder, core: &mut ShellCore) -> Option<VarName> {
        match feeder.scanner_name(core) {
            0 => None,
            n => Some(VarName {
                text: feeder.consume(n),
            }),
        }
    }

    fn eat_char(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        if feeder.starts_with(")") {
            ans.text += &feeder.consume(1);
            ans.subwords.push(From::from(")"));
            return Ok(false);
        }

        let len = feeder.scanner_char();
        if len == 0 {
            feeder.feed_additional_line(core)?;
            return Ok(true);
        }

        let ch = feeder.consume(len);
        ans.text += &ch.clone();
        ans.subwords.push(From::from(&ch));
        Ok(true)
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        mode: &Option<WordMode>,
    ) -> Result<Option<Self>, ParseError> {
        match mode {
            Some(WordMode::EvalLet) => {}
            _ => return Ok(None),
        }

        if !feeder.starts_with("(") {
            return Ok(None);
        }

        let mut ans = Self::default();

        while Self::eat_element(feeder, &mut ans, core)? || Self::eat_char(feeder, &mut ans, core)?
        {
        }

        Ok(Some(ans))
    }
}
