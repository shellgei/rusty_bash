//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use crate::{Feeder, Script, ShellCore};
use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct ParenCommand {
    text: String,
    script: Option<Script>,
    redirects: Vec<Redirect>,
    lineno: usize,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        self.fork_exec(core, pipe)
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) -> Result<(), ExecError> {
        if !fork {
            exit::internal(" (no fork for subshell)");
        }

        match self.script {
            Some(ref mut s) => s.exec(core)?,
            _ => exit::internal(" (ParenCommand::exec)"),
        }

        Ok(())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self.redirects
    }
    fn get_lineno(&mut self) -> usize {
        self.lineno
    }
    fn set_force_fork(&mut self) {}
    fn boxed_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn force_fork(&self) -> bool {
        true
    }

    fn get_one_line_text(&self) -> String {
        return match &self.script {
            Some(s) => format!("( {} )", s.get_one_line_text()),
            None => "()".to_string(),
        };
    }
}

impl ParenCommand {
    /*
        pub fn new(text: &str, script: Option<Script>) -> Self {
            Self {
                text: text.to_string(),
                script: script,
                redirects: vec![],
            }
        }
    */
    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        substitution: bool,
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        if command::eat_inner_script(feeder, core, "(", vec![")"], &mut ans.script, substitution)? {
            ans.text.push_str("(");
            ans.text.push_str(&ans.script.as_ref().unwrap().get_text());
            ans.text.push_str(&feeder.consume(1));

            if !substitution {
                command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            }
            Ok(Some(ans))
        } else {
            Ok(None)
        }
    }
}
