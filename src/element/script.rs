//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::operators::*;
use crate::element::command::CommandType;
use crate::element::pipeline::Pipeline;
use crate::utils::blue_string;

pub struct Script {
    pub list: Vec<Pipeline>,
    pub list_ends: Vec<ControlOperator>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, conf: &mut ShellCore) {
        let mut eop = ControlOperator::NoChar;
        for (i, p) in self.list.iter_mut().enumerate() {
            if conf.has_flag('d') {
                eprintln!("{}", blue_string(&p.get_text()));
            }

            let status = conf.get_var("?") == "0";
           
            if (status && eop == ControlOperator::Or) || (!status && eop == ControlOperator::And) {
                eop = self.list_ends[i].clone();
                continue;
            }
            p.exec(conf);
            if conf.return_flag {
                conf.return_flag = false;
                return;
            }
            eop = self.list_ends[i].clone();
        }
    }

    pub fn new() -> Script{
        Script {
            list: vec![],
            list_ends: vec![],
            text: "".to_string(),
        }
    }

    fn is_end_condition(parent: &CommandType, op: &ControlOperator) -> bool {
        ( parent == &CommandType::Paren && op == &ControlOperator::RightParen ) ||
        ( parent == &CommandType::Case && op == &ControlOperator::DoubleSemicolon )
    }

    fn set_listend(text: &mut Feeder, ans: &mut Script, parent_type: &CommandType) -> bool {
        let (n, op) = text.scanner_control_op();
        if let Some(p) = op {
            ans.list_ends.push(p.clone());
            if Script::is_end_condition(parent_type, &p) {
                return true;
            }

            ans.text += &text.consume(n);
        }else{
            ans.list_ends.push(ControlOperator::NoChar);
        }

        false
    }

    fn read_blank(text: &mut Feeder, ans: &mut Script) {
        loop {
            let before = ans.text.len();
            ans.text += &text.consume_blank_return();
            ans.text += &text.consume_comment();

            if before == ans.text.len() || text.len() == 0 {
                return;
            }
        }
    }

    pub fn parse_elem(text: &mut Feeder, conf: &mut ShellCore, ans: &mut Script, parent_type: &CommandType) -> bool {
        let mut go_next = true;

        if let Some(result) = Pipeline::parse(text, conf) {
            ans.text += &result.text;
            ans.list.push(result);

            if Script::set_listend(text, ans, parent_type){
                go_next = false;
            }
        }
        else {
            go_next = false;
        }

        go_next
    }

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
        Script::read_blank(text, &mut ans);
        while  Script::parse_elem(text, conf, &mut ans, parent_type) {
            /* If a semicolon exist, another element can be added to the list */
            let (n, op) = text.scanner_control_op();
            if op == Some(ControlOperator::Semicolon) {
                ans.text += &text.consume(n);
            }

            if text.len() == 0 && parent_type == &CommandType::Null {
                break;
            }

            text.request_next_line(conf);
            Script::read_blank(text, &mut ans);
        }

        if ans.text.len() > 0 {
            Some(ans)
        }else{
            eprintln!("Unknown phrase");
            conf.set_var("?", "1");
            text.consume(text.len());
            None
        }
    }
}
