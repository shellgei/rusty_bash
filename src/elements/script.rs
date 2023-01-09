//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::CommandType;
use crate::elements::job::Job;
//use crate::operators::ControlOperator;

#[derive(Debug)]
pub struct Script {
    pub list: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, conf: &mut ShellCore) {
        for j in self.list.iter_mut() {
            j.exec(conf);

            if conf.return_flag {
                conf.return_flag = false;
                return;
            }
        }
    }

    pub fn new() -> Script{
        Script {
            list: vec![],
            text: "".to_string(),
        }
    }

    /*
    fn is_end_condition(parent: &CommandType, op: &ControlOperator) -> bool {
        ( parent == &CommandType::Paren && op == &ControlOperator::RightParen ) ||
        ( parent == &CommandType::Case && op == &ControlOperator::DoubleSemicolon )
    }*/

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore,
                 parent_type: &CommandType) -> Option<Script> {
        if text.len() == 0 {
            return None;
        };
    
        if text.starts_with(")") {
            eprintln!("Unexpected symbol: {}", text.consume(text.len()).trim_end());
            conf.set_var("?", "2");
            return None;
        }

        let mut ans = Script::new();
        loop {
            ans.text += &text.consume_blank();
            if let Some(j) =  Job::parse(text, conf, parent_type) {
                ans.text += &j.text.clone();
                ans.list.push(j);
            }else{
                break;
            }
        }

        if ans.list.len() > 0 {
            Some( ans )
        }else{
            None
        }
    }
}
