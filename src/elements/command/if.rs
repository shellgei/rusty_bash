//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::elements::command;
use crate::utils::exit;
use super::{Command, Redirect};

#[derive(Debug, Clone, Default)]
pub struct IfCommand {
    pub text: String,
    pub if_elif_scripts: Vec<Script>,
    pub then_scripts: Vec<Script>,
    pub else_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for IfCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool, feeder: &mut Feeder) -> Result<(), ExecError> {
        for i in 0..self.if_elif_scripts.len() {
            self.if_elif_scripts[i].exec(core, feeder)?;
            if core.db.exit_status == 0 {
                let _ = self.then_scripts[i].exec(core, feeder);
                return Ok(());
            }else {
                core.db.exit_status = 0;
            }
        }

        match self.else_script.as_mut() {
            Some(s) => s.exec(core, feeder)?,
            _ => {},
        }
        Ok(())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl IfCommand {
    fn end_words(word: &str) -> Result<Vec<&str>, ParseError> {
        match word {
            "if" | "elif" => Ok(vec!["then"]),
            "then" => Ok(vec!["fi", "else", "elif"]),
            "else" => Ok(vec!["fi"]),
            unknown => return Err(ParseError::UnexpectedSymbol(unknown.to_string())),
        }
    }

    fn set_script(word: &str, ans: &mut IfCommand, script: Option<Script>) {
        match word {
            "if" | "elif" => ans.if_elif_scripts.push(script.unwrap()),
            "then"        => ans.then_scripts.push(script.unwrap()),
            "else"        => ans.else_script = script,
            _ => exit::internal(" (if parse error)"),
        };
    }

    fn eat_word_and_script(word: &str, feeder: &mut Feeder,
                           ans: &mut IfCommand, core: &mut ShellCore) -> Result<bool, ParseError> {
        let mut s = None;
        let ends = Self::end_words(word)?;
        if ! command::eat_inner_script(feeder, core, word, ends, &mut s, false)? {
            return Ok(false);
        }

        ans.text.push_str(word);
        ans.text.push_str(&s.as_ref().unwrap().get_text());
        Self::set_script(word, ans, s);
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
 
        let mut if_or_elif = "if";
        while Self::eat_word_and_script(if_or_elif, feeder, &mut ans, core)?
           && Self::eat_word_and_script("then", feeder, &mut ans, core)? {

            Self::eat_word_and_script("else", feeder, &mut ans, core)?; //optional

            if feeder.starts_with("fi") { // If "else" exists, always it comes here.
                ans.text.push_str(&feeder.consume(2));
                break;
            }

            if_or_elif = "elif";
        }

        if ans.then_scripts.is_empty() {
            return Ok(None);
        }

        command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
        Ok(Some(ans))
    }
}
