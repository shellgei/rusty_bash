//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use crate::elements::command;

#[derive(Debug, Clone, Default)]
pub struct WhileCommand {
    pub text: String,
    pub while_script: Option<Script>,
    pub do_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
    lineno: usize,
}

impl Command for WhileCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        if core.return_flag {
            return Ok(());
        }
        core.loop_level += 1;
        while ! core.return_flag {
            core.suspend_e_option = true;
            self.while_script.clone().as_mut().unwrap().exec(core)?;

            core.suspend_e_option = false;
            if core.db.exit_status != 0 {
                core.db.exit_status = 0;
                break;
            }

            if core.continue_counter > 0 {
                core.continue_counter -= 1;
                continue;
            }

            self.do_script.clone().as_mut().unwrap().exec(core)?;

            if core.break_counter > 0 {
                core.break_counter -= 1;
                break;
            }
        }
        core.loop_level -= 1;
        if core.loop_level == 0 {
            core.break_counter = 0;
        }
        Ok(())
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn get_lineno(&mut self) -> usize { self.lineno }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl WhileCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        if ! command::eat_inner_script(feeder, core, "while", vec!["do"],
                                       &mut ans.while_script, false)?{
            return Ok(None);
        }
        while command::eat_blank_with_comment(feeder, core, &mut ans.text) {}

        if command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script, false)? {
            ans.text.push_str("while");
            ans.text.push_str(&ans.while_script.as_mut().unwrap().get_text());
            ans.text.push_str("do");
            ans.text.push_str(&ans.do_script.as_mut().unwrap().get_text());
            ans.text.push_str(&feeder.consume(4)); //done

            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            Ok(Some(ans))
        }else{
            Ok(None)
        }
    }
}
