//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::utils;
use super::SimpleCommand;

impl SimpleCommand {
    pub fn eat_substitution(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        if let Some(s) = Substitution::parse(feeder, core)? {
            self.text += &s.text;
            self.substitutions.push(s);
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let w = match Word::parse(feeder, core)? {
            Some(w) => w,
            _       => return Ok(false),
        };

        if ans.words.is_empty() && utils::reserved(&w.text) {
            return Ok(false);
        }
        ans.text += &w.text;
        ans.words.push(w);
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        loop { 
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            if ! ans.eat_substitution(feeder, core)? {
                break;
            }
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            if ! Self::eat_word(feeder, &mut ans, core)? {
                break;
            }
        }

        if ans.words.len() + ans.redirects.len() + ans.substitutions.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
