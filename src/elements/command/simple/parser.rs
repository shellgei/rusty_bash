//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{alias, SimpleCommand, SubsArgType};
use crate::elements::command;
use crate::elements::command::{Command, ParenCommand};
use crate::elements::script::Script;
use crate::elements::substitution::Substitution;
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use crate::{utils, Feeder, ShellCore};
use crate::elements::pipeline::Pipeline;
use crate::elements::job::Job;

impl SimpleCommand {
    pub fn eat_substitution(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        match Substitution::parse(feeder, core, false) {
            Ok(Some(s)) => {
                ans.text += &s.text;
                ans.substitutions.push(s);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                feeder.rewind();
                Err(e)
            }
        }
    }

    pub fn eat_substitution_as_arg(
        feeder: &mut Feeder,
        ans: &mut Self,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        if let Some(s) = Substitution::parse_as_arg(feeder, core)? {
            ans.text += &s.text;
            ans.substitutions_as_args
                .push(SubsArgType::Subs(Box::new(s)));
            return Ok(true);
        }

        if let Some(w) = Word::parse(feeder, core, None)? {
            ans.text += &w.text;
            ans.substitutions_as_args.push(SubsArgType::Other(w));
            return Ok(true);
        }

        Ok(false)
    }

    fn eat_word(
        feeder: &mut Feeder,
        ans: &mut SimpleCommand,
        core: &mut ShellCore,
    ) -> Result<bool, ParseError> {
        let mut mode = None;
        if ans.command_name == "eval" || ans.command_name == "let" {
            mode = Some(WordMode::EvalLet);
        }

        let w = match Word::parse(feeder, core, mode) {
            Ok(Some(w)) => w,
            Err(e) => {
                feeder.rewind();
                return Err(e);
            }
            _ => return Ok(false),
        };

        if ans.words.is_empty() {
            ans.lineno = feeder.lineno;
            if utils::reserved(&w.text) {
                return Ok(false);
            }

            ans.command_name = w.text.clone();
        }

        if (ans.words.is_empty() || ans.continue_alias_check) && alias::set(ans, &w, core, feeder)?
        {
            return Ok(true);
        }

        ans.text += &w.text;
        ans.words.push(w);

        Ok(true)
    }

    fn make_dummy_paren(&mut self) -> Result<Option<Box<dyn Command>>, ParseError> {
        let mut pip = Pipeline::default();
        pip.text = self.text.clone();
        pip.commands.push(self.boxed_clone());

        let mut job = Job::default();
        job.text = pip.text.clone();
        job.pipelines.push(pip);
        job.pipeline_ends.push("".to_string());

        let mut script = Script::default();
        script.text = job.text.clone();
        script.jobs.push(job);
        script.job_ends.push("".to_string());

        let mut com = ParenCommand::default();
        com.lineno = self.lineno;
        com.text = script.text.clone();
        com.script = Some(script);

        Ok(Some(Box::new(com)))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Box<dyn Command>>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        loop {
            let exist_redirect =
                match command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text) {
                    Ok(exist) => exist,
                    Err(e) => {
                        feeder.rewind();
                        return Err(e);
                    }
                };

            let exist_sub = match Self::eat_substitution(feeder, &mut ans, core) {
                Ok(exist) => exist,
                Err(e) => {
                    feeder.rewind();
                    return Err(e);
                }
            };

            if !exist_redirect && !exist_sub {
                break;
            }
        }

        loop {
            if let Err(e) = command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)
            {
                feeder.rewind();
                return Err(e);
            }

            if core.substitution_builtins.contains_key(&ans.command_name)
                && Self::eat_substitution_as_arg(feeder, &mut ans, core)?
            {
                continue;
            }

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if !Self::eat_word(feeder, &mut ans, core)? {
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
                return ans.make_dummy_paren();
            }

            Ok(Some(Box::new(ans)))
        } else {
            feeder.rewind();
            Ok(None)
        }
    }
}
