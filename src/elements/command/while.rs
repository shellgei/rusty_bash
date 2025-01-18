//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::error::parse::ParseError;
use super::{Command, Redirect};
use crate::elements::command;

#[derive(Debug, Default)]
pub struct WhileCommand {
    pub text: String,
    pub while_script: Option<Script>,
    pub do_script: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        loop {
            self.while_script.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core);

            if core.db.get_param("?").unwrap() != "0" {
                break;
            }

            self.do_script.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core);
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn force_fork(&self) -> bool { self.force_fork }
}

impl WhileCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        if command::eat_inner_script(feeder, core, "while", vec!["do"], &mut ans.while_script)?
        && command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.do_script)? {
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
