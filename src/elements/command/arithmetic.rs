//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::arithmetic_expression::ArithmeticExpr;

#[derive(Debug, Clone)]
pub struct ArithmeticCommand {
    text: String,
    arith: Vec<ArithmeticExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for ArithmeticCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        let mut ans = true;

        for a in &mut self.arith {
            match a.eval(core) {
                Some(s) => ans = s != "0",
                None    => {
                    ans = false;
                    break;
                },
            }
        }

        core.data.set_param("?", if ans {"0"} else {"1"} );
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
            arith: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("((") {
            return None;
        }
        feeder.set_backup();

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        loop {
            if let Some(c) = ArithmeticExpr::parse(feeder, core) {
                if feeder.starts_with(",") {
                    ans.text += &c.text;
                    ans.text += &feeder.consume(1);
                    ans.arith.push(c);
                    continue;
                }

                if feeder.starts_with("))") {
                    ans.text += &c.text;
                    ans.text += &feeder.consume(2);
                    ans.arith.push(c);
                    feeder.pop_backup();
                    return Some(ans);
                }
    
                break;
            }else{
                break;
            }
        }
        feeder.rewind();
        return None;
    }
}
