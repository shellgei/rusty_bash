//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::command::SimpleCommand;
use crate::elements::pipeline::Pipeline;
use crate::elements::job::Job;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use crate::{Feeder, Script, ShellCore};
use nix::unistd::Pid;

impl From<SimpleCommand> for ParenCommand {
    fn from(c: SimpleCommand) -> Self {
        let mut pip = Pipeline::default();
        pip.text = c.get_text();
        pip.commands.push(c.boxed_clone());

        let mut job = Job::default();
        job.text = pip.text.clone();
        job.pipelines.push(pip);
        job.pipeline_ends.push("".to_string());

        let mut script = Script::default();
        script.text = job.text.clone();
        script.jobs.push(job);
        script.job_ends.push("".to_string());

        let mut com = ParenCommand::default();
        com.lineno = c.lineno;
        com.text = script.text.clone();
        com.script = Some(script);

        com
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParenCommand {
    pub text: String,
    pub script: Option<Script>,
    redirects: Vec<Redirect>,
    pub lineno: usize,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        if core.break_counter > 0 || core.continue_counter > 0 {
            return Ok(None);
        }

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
        match &self.script {
            Some(s) => format!("( {} )", s.get_one_line_text()),
            None => "()".to_string(),
        }
    }
}

impl ParenCommand {
    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        substitution: bool,
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self {
            lineno: feeder.lineno,
            ..Default::default()
        };

        if command::eat_inner_script(feeder, core, "(", vec![")"], &mut ans.script, substitution)? {
            ans.text.push('(');
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
