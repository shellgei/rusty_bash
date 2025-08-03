//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::job::Job;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct RepeatCommand {
    pub text: String,
    times: Word,
    job: Job,
    force_fork: bool,
    lineno: usize,
    _dummy: Vec<Redirect>,
}

impl Command for RepeatCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        let mut f = Feeder::new(&self.times.text);

        let n = match ArithmeticExpr::parse(&mut f, core, false, "")? {
            Some(mut a) => a.eval(core)?.parse::<usize>()?,
            None => 0,
        };

        for _ in 0..n {
            self.job.clone().exec(core, false)?;
        }

        Ok(())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self._dummy
    }
    fn get_lineno(&mut self) -> usize {
        self.lineno
    }
    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
    fn boxed_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn force_fork(&self) -> bool {
        self.force_fork
    }
}

impl RepeatCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("repeat") {
            return Ok(None);
        }

        let mut ans = Self {
            lineno: feeder.lineno,
            text: feeder.consume(6),
            ..Default::default()
        };
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        ans.times = match Word::parse(feeder, core, None)? {
            Some(w) => w,
            _ => return Err(ParseError::UnexpectedSymbol("repeat".to_string())),
        };
        let _ = command::eat_blank_lines(feeder, core, &mut ans.text);

        ans.job = match Job::parse(feeder, core)? {
            Some(j) => j,
            _ => return Err(ParseError::UnexpectedSymbol("repeat".to_string())),
        };

        Ok(Some(ans))
    }
}
