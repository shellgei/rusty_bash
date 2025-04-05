//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::exec::ExecError;

#[derive(Debug, Clone)]
pub struct ArithmeticCommand {
    pub text: String,
    expressions: Vec<ArithmeticExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for ArithmeticCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        let exit_status = match self.eval(core).as_deref() {
            Ok("0") => 1,
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{:?}", e);
                1
            },
        };
        core.db.exit_status = exit_status;
        Ok(())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl ArithmeticCommand {
    fn new() -> ArithmeticCommand {
        ArithmeticCommand {
            text: String::new(),
            expressions: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn eval(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        if core.db.flags.contains('x') {
            let ps4 = core.get_ps4();
            eprintln!("\r{} {}\r", ps4, &self.text);
        }

        let mut ans = String::new();
        for a in &mut self.expressions {
            ans = a.eval(core)?;
        }

        Ok(ans)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("((") {
            return Ok(None);
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        if let Some(c) = ArithmeticExpr::parse(feeder, core, true, "((")? {
                dbg!("{:?}", &c);
                dbg!("{:?}", &feeder);
            if feeder.starts_with("))") {
                ans.text += &c.text;
                ans.text += &feeder.consume(2);
                ans.expressions.push(c);
                feeder.pop_backup();
                dbg!("{:?}", &ans);
                return Ok(Some(ans));
            }
        }
        feeder.rewind();
        return Ok(None);
    }
}
