//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{alias, SimpleCommand, SubsArgType};
use crate::elements::command;
use crate::elements::command::{Command, ParenCommand};
use crate::elements::substitution::Substitution;
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use crate::{utils, Feeder, ShellCore};

impl SimpleCommand {
    pub fn eat_substitution(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        match Substitution::parse(feeder, core, false, false)? {
            Some(s) => {
                self.text += &s.text;
                self.substitutions.push(s);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn eat_substitution_as_arg(&mut self, feeder: &mut Feeder,core: &mut ShellCore)
    -> Result<bool, ParseError> {
        //if let Some(s) = Substitution::parse_as_arg(feeder, core)? {
        if let Some(s) = Substitution::parse(feeder, core, false, true)? {
            self.text += &s.text;
            self.substitutions_as_args
                .push(SubsArgType::Subs(Box::new(s)));
            return Ok(true);
        }

        if let Some(w) = Word::parse(feeder, core, None)? {
            self.text += &w.text;
            self.substitutions_as_args.push(SubsArgType::Other(w));
            return Ok(true);
        }

        Ok(false)
    }

    fn eat_word(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        let mut mode = None;
        if self.command_name == "eval" || self.command_name == "let" {
            mode = Some(WordMode::EvalLet);
        }

        let w = match Word::parse(feeder, core, mode)? {
            Some(w) => w,
            _ => return Ok(false),
        };

        if self.words.is_empty() {
            self.lineno = feeder.lineno;
            if utils::reserved(&w.text) {
                return Ok(false);
            }

            self.command_name = w.text.clone();
        }

        if (self.words.is_empty()
            || self.continue_alias_check)
            && alias::set(self, &w, core, feeder)?  {
            return Ok(true);
        }

        self.text += &w.text;
        self.words.push(w);

        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Box<dyn Command>>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        while command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?
              || ans.eat_substitution(feeder, core)? {}

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;

            if core.subst_builtins.contains_key(&ans.command_name) {
                if ans.eat_substitution_as_arg(feeder, core)? {
                    continue;
                }
            }

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! ans.eat_word(feeder, core)? {
                break;
            }
        }

        if ans.invalid_alias {
            feeder.pop_backup();
            feeder.consume(feeder.len());
            return Ok(None);
        }

        if ans.substitutions.len() + ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();

            if ans.words.iter_mut().any(|w| w.is_to_proc_sub() ) {
                return Ok(Some(Box::new(ParenCommand::from(ans))));
            }

//            ans.read_heredoc(feeder, core)?;//TODO: maybe required for every command
            Ok(Some(Box::new(ans)))
        } else {
            feeder.rewind();
            Ok(None)
        }
    }
}
