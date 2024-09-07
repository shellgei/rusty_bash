//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::conditional_expr::ConditionalExpr;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
enum Elem {
    FileCheckOption(String),
    Word(Word),
    /*
    RightParen,
    LeftParen,
    Not,  // ! 
    And,  // &&
    Or,  // ||
    Expression(ConditionalExpr),
    */
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

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match Word::parse(feeder, core, false) {
            Some(w) => {
                ans.text += &w.text.clone();
                ans.elements.push(Elem::Word(w));

                true
            },
            _ => false
        }
    }

    fn eat_file_check_option(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_file_check_option(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(Elem::FileCheckOption(opt));

        true
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.scanner_blank(core) {
            0 => false,
            n => {
                ans.text += &feeder.consume(n);
                true
            },
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);

        loop {
            if ! Self::eat_blank(feeder, &mut ans, core) {
                return None;
            }
            if feeder.starts_with("]]") {
                ans.text += &feeder.consume(2);
                command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);

                dbg!("{:?}", &ans);
                return match ans.elements.len() > 0 {
                    true  => Some(ans),
                    false => None,
                };
            }

            if Self::eat_file_check_option(feeder, &mut ans, core) 
            || Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            break;
        }
    
        None
    }
}
