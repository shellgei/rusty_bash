//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::expr::conditional::ConditionalExpr;
use crate::elements::expr::conditional::elem::CondElem;

#[derive(Debug, Clone, Default)]
pub struct TestCommand {
    text: String,
    cond: Option<ConditionalExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        if core.db.flags.contains('x') {
            let ps4 = core.get_ps4();
            eprint!("\r{} [[ ", &ps4);
        }

        match self.cond.clone().unwrap().eval(core) {
            Ok(CondElem::Ans(true))  => core.db.exit_status = 0,
            Ok(CondElem::Ans(false)) => core.db.exit_status = 1,
            Err(err_msg)  => {
                core.db.exit_status = 2;
                return Err(err_msg);
            },
            _  => {
                core.db.exit_status = 2;
                return Err(ExecError::Other("unknown error".to_string()));
            },
        } ;
        Ok(())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("[[") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text = feeder.consume(2);

        command::eat_blank_lines(feeder, core, &mut ans.text)?;

        match ConditionalExpr::parse(feeder, core) {
            Some(e) => {
                ans.text += &e.text.clone();
                ans.cond = Some(e);
            },
            None => return Ok(None),
        }

        command::eat_blank_lines(feeder, core, &mut ans.text)?;

        if feeder.starts_with("]]") {
            ans.text += &feeder.consume(2);
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            return Ok(Some(ans));
        }
    
        Ok(None)
    }
}
