//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::element_list::*;
use crate::elements::function::Function;
use crate::elements::pipeline::Pipeline;
use crate::utils::blue_string;
use crate::ListElem;
use crate::scanner::scanner_control_op;

pub struct Script {
    pub list: Vec<Box<dyn ListElem>>,
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

    fn is_end_condition(parent: &CompoundType, op: &ControlOperator) -> bool {
        ( parent == &CompoundType::Paren && op == &ControlOperator::RightParen ) ||
        ( parent == &CompoundType::Case && op == &ControlOperator::DoubleSemicolon )
    }

    fn set_listend(text: &mut Feeder, ans: &mut Script, parent_type: &CompoundType) -> bool {
        let (n, op) = scanner_control_op(text);
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

    pub fn parse_elem(text: &mut Feeder, conf: &mut ShellCore, ans: &mut Script, parent_type: &CompoundType) -> (bool, bool) {
        let mut is_function = false;
        let mut go_next = true;

        if let Some(f) = Function::parse(text, conf) {
            ans.text += &f.text;
            let body = f.body.get_text();
            conf.functions.insert(f.name, body);
            is_function = true;
        }else if let Some(result) = Pipeline::parse(text, conf) {
            ans.text += &result.text;
            ans.list.push(Box::new(result));

            if Script::set_listend(text, ans, parent_type){
                go_next = false;
            }
        }
        else {
            go_next = false;
        }

        (go_next, is_function)
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore,
                 parent_type: &CompoundType) -> Option<Script> {
        if text.len() == 0 {
            return None;
        };
    
        if text.nth(0) == ')' {
            eprintln!("Unexpected symbol: {}", text.consume(text.len()).trim_end());
            conf.set_var("?", "2");
            return None;
        }

        let mut ans = Script::new();
        let mut read_function = false;
    
        loop {
            Script::read_blank(text, &mut ans);

            /* read a function, pipeline, or variable setting */
            let (go_next, is_func) = Script::parse_elem(text, conf, &mut ans, parent_type);
            read_function |= is_func;

            if ! go_next {
                break;
            }

            /* If a semicolon exist, another element can be added to the list */
            let (n, op) = scanner_control_op(text);
            if op == Some(ControlOperator::Semicolon) {
                ans.text += &text.consume(n);
            }

            if text.len() == 0 && parent_type == &CompoundType::Null {
                break;
            }

            text.request_next_line(conf);
        }

        if ans.text.len() > 0 || read_function {
            Some(ans)
        }else{
            eprintln!("Unknown phrase");
            conf.set_var("?", "1");
            text.consume(text.len());
            None
        }
    }
}
