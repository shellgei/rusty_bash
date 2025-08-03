//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Redirect};
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

#[derive(Debug, Clone, Default)]
pub struct ArithmeticCommand {
    pub text: String,
    expressions: Vec<ArithmeticExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
    lineno: usize,
}

impl Command for ArithmeticCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        let mut err = None;

        let exit_status = match self.eval(core) {
            Ok(n) => {
                if n == "0" {
                    1
                } else {
                    0
                }
            }
            Err(e) => {
                err = Some(e);
                1
            }
        };

        core.db.exit_status = exit_status;

        match err {
            Some(ExecError::ArithError(s, e)) => {
                let err_with_com = ExecError::ArithError("((: ".to_owned() + s.trim_start(), e);
                err_with_com.print(core);
                Err(err_with_com)
            }
            Some(e) => {
                e.print(core);
                Err(e.clone())
            }
            _ => Ok(()),
        }
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

impl ArithmeticCommand {
    /*
    fn new() -> ArithmeticCommand {
        ArithmeticCommand {
            text: String::new(),
            expressions: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }*/

    pub fn eval(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut ans = String::new();
        for a in &mut self.expressions {
            ans = a.eval(core)?;
        }

        Ok(ans)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("((") {
            return Ok(None);
        }
        feeder.set_backup();

        let mut ans = Self {
            lineno: feeder.lineno,
            text: feeder.consume(2),
            ..Default::default()
        };

        if let Some(c) = ArithmeticExpr::parse(feeder, core, true, "((")? {
            if feeder.starts_with("))") {
                ans.text += &c.text;
                ans.text += &feeder.consume(2);
                ans.expressions.push(c);
                feeder.pop_backup();
                return Ok(Some(ans));
            }
        }
        feeder.rewind();
        Ok(None)
    }
}
