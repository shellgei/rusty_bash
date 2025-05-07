//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, utils};
use super::SimpleCommand;
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::error::parse::ParseError;

impl SimpleCommand {
    fn eat_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        if let Some(s) = Substitution::parse(feeder, core)? {
            ans.text += &s.text;
            match ans.command_name.as_ref() {
                "local" | "eval" | "export"  => ans.substitutions_as_args.push(s),
                _ => ans.substitutions.push(s),
            }
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            Err(e) => {
                feeder.rewind();
                return Err(e);
            },
            _       => {
                return Ok(false);
            },
        };

        if ans.words.is_empty() {
            ans.lineno = feeder.lineno;
            if utils::reserved(&w.text) {
                return Ok(false);
            }

            ans.command_name = w.text.clone();
        }

        if ans.words.is_empty() || ans.continue_alias_check {
            if ans.set_alias(&w, core, feeder)? {
                return Ok(true);
            }
        }

        ans.text += &w.text;
        ans.words.push(w);

        Ok(true)
    }

    fn set_alias(&mut self, word: &Word,
                 core: &mut ShellCore, feeder: &mut Feeder) -> Result<bool, ParseError> {
        self.continue_alias_check = false;
        let mut w = word.text.clone();
        if ! core.replace_alias(&mut w) {
            return Ok(false);
        }

        self.continue_alias_check = w.ends_with(" ");

        let mut feeder_local = Feeder::new(&mut w);

        loop {
            if let Some(s) = Substitution::parse(&mut feeder_local, core)? {
                self.text += &s.text;
                match self.command_name.as_ref() {
                    "local" | "eval" | "export"  => self.substitutions_as_args.push(s),
                    _ => self.substitutions.push(s),
                }
                command::eat_blank_with_comment(&mut feeder_local, core, &mut self.text);
            }else{
                break;
            }
        }

        loop {
            match Word::parse(&mut feeder_local, core, None) {
                Ok(Some(w)) => {
                    self.text.push_str(&w.text);
                    self.words.push(w);
                },
                _    => break,
            }
            command::eat_blank_with_comment(&mut feeder_local, core, &mut self.text);
        }

        if self.words.is_empty() && self.substitutions.is_empty() {
            return Err(ParseError::WrongAlias(w));
        }

        feeder.replace(0, &feeder_local.consume(feeder_local.len()));
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        while command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?
        || Self::eat_substitution(feeder, &mut ans, core)? {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            if ( ans.command_name == "local" || ans.command_name == "eval" || ans.command_name == "export")
            && Self::eat_substitution(feeder, &mut ans, core)? {
                continue;
            }

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! Self::eat_word(feeder, &mut ans, core)? {
                break;
            }
        }

        if ans.substitutions.len() + ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
