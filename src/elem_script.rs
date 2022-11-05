//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::element_list::ControlOperator;
use crate::elem_function::Function;
use crate::elem_pipeline::Pipeline;
use crate::elem_setvars::SetVariables;
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

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, end: Vec<&str>) -> Option<Script> {
        if text.len() == 0 {
            return None;
        };
    
        if text.nth(0) == ')' {
            eprintln!("Unexpected symbol: {}", text.consume(text.len()).trim_end());
            conf.set_var("?", "2");
            return None;
        }

        let mut ans = Script::new();
        let mut is_function = false;
    
        loop {
            loop {
                let before = ans.text.len();
                ans.text += &text.consume_blank_return();
                ans.text += &text.consume_comment();

                if before == ans.text.len() || text.len() == 0 {
                    break;
                }
            }

            if let Some(f) = Function::parse(text, conf) {
                ans.text += &f.text;
                let body = f.body.get_text();
                conf.functions.insert(f.name, body);
                is_function = true;
            }else if let Some(result) = SetVariables::parse(text, conf) {
                ans.list_ends.push(result.get_end());
                ans.text += &result.text;
                ans.list.push(Box::new(result));

                let (n, op) = scanner_control_op(text, 0);
                if let Some(p) = op {
                    ans.text += &text.consume(n);
                }

                if end.len() == 1 && end[0] == ";;"  {
                    if let Some(op) = ans.list_ends.last() {
                        if op == &ControlOperator::DoubleSemicolon {
                            break;
                        }
                    }
                }
            }else if let Some(result) = Pipeline::parse(text, conf) {

                ans.text += &result.text;
                ans.list.push(Box::new(result));

                let (n, op) = scanner_control_op(text, 0);
                ans.text += &text.consume(n);
                if let Some(p) = op {
                    ans.list_ends.push(p);
                }else{
                    ans.list_ends.push(ControlOperator::NoChar);
                }
        
                if end.len() == 1 && end[0] == ";;"  {
                    if let Some(op) = ans.list_ends.last() {
                        if op == &ControlOperator::DoubleSemicolon {
                            break;
                        }
                    }
                }

            }
            else {
                break
            }

            //TODO: this removal of control operator should be on one more upper level.
            let (n, op) = scanner_control_op(text, 0);
            if let Some(p) = op {
                ans.text += &text.consume(n);
            }

            if let Some(op) = ans.list_ends.last() {
                if op == &ControlOperator::DoubleSemicolon {
                    break;
                }
            }


            if text.len() == 0 && end[0] == "" {
                break;
            }

            if end.iter().any(|e| text.compare(0, e)) {
                break;
            }

            if text.len() > 0 && text.nth(0) == ')'  {
                break;
            }else{
                text.request_next_line(conf);
            }
        }
    
        if ans.text.len() > 0 || is_function {
            Some(ans)
        }else{
            eprintln!("Unknown phrase");
            conf.set_var("?", "1");
            text.consume(text.len());
            None
        }
    }
}
