//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::conditional_expr::ConditionalExpr;

#[derive(Debug, Clone)]
enum Elem {
    RightParen,
    LeftParen,
    Not,  // ! 
    And,  // &&
    Or,  // ||
    Expression(ConditionalExpr),
}

#[derive(Debug, Clone)]
pub struct TestCommand {
    text: String,
    elements: Vec<Elem>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        core.data.set_param("?", "0");
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    fn new() -> TestCommand {
        TestCommand {
            text: String::new(),
            elements: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        while ! feeder.starts_with("]]") {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            match ConditionalExpr::parse(feeder, core) {
                Some(c) => {
                    ans.text += &c.text;
                    match c.expr {
                        None => return None,
                        _ => ans.elements.push(Elem::Expression(c)),
                    }
                }
                None => {},
            }
        }
    
        ans.text += &feeder.consume(2);
        command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);

        dbg!("{:?}", &ans);
        match ans.elements.len() > 0 {
            true  => Some(ans),
            false => None,
        }
    }
}
