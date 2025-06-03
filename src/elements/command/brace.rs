//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use super::{Command, Redirect};
use crate::elements::command;

#[derive(Debug, Clone, Default)]
pub struct BraceCommand {
    text: String,
    script: Option<Script>,
    redirects: Vec<Redirect>,
    force_fork: bool,
    lineno: usize,
}

impl Command for BraceCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        match self.script {
            Some(ref mut s) => s.exec(core)?,
            _ => exit::internal(" (ParenCommand::exec)"),
        }
        Ok(())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn get_lineno(&mut self) -> usize { self.lineno }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }

    fn pretty_print(&mut self, indent_num: usize) {
        println!("{} ", "{");
        for s in self.script.iter_mut() {
            s.pretty_print(indent_num+1);
        }
        println!("{}", "}");
    }
}

impl BraceCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        ans.lineno = feeder.lineno;
        if command::eat_inner_script(feeder, core, "{", vec!["}"], &mut ans.script, false)? {
            ans.text.push_str("{");
            ans.text.push_str(&ans.script.as_ref().unwrap().get_text());
            ans.text.push_str(&feeder.consume(1));

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            Ok(Some(ans))
        }else{
            Ok(None)
        }
    }
}
