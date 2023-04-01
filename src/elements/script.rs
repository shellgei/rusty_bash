//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::job::Job;

enum EndStatus{
    UnexpectedSymbol(String),
    NeedMoreLine,
    NormalEnd,
}

#[derive(Debug)]
pub struct Script {
    pub list: Vec<Job>,
    pub text: String,
}

impl Script {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for j in self.list.iter_mut() {
            j.exec(core);

            if core.return_flag {
                core.return_flag = false;
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
    fn unexpected_symbol(feeder: &mut Feeder) -> bool {
        if feeder.len() == 0 {
            return false;
        }

        if let Some(_) =  ")}".find(feeder.nth(0)){
            return true;
        }

        false
    }*/

    fn eat_job(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Script) -> bool { 
        ans.text += &feeder.consume_blank();
        if let Some(j) =  Job::parse(feeder, core) {
            ans.text += &j.text.clone();
            ans.list.push(j);
            true
        }else{
            false
        }
    }

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore, empty: bool) -> EndStatus {
        if let Some(begin) = core.nest.pop() {
            core.nest.push(begin.clone());
            if begin == "(" {
                if feeder.starts_with(")"){
                    if empty {
                        return EndStatus::UnexpectedSymbol(")".to_string());
                    }
                    return EndStatus::NormalEnd;
                }else if feeder.starts_with("}"){
                    return EndStatus::UnexpectedSymbol("}".to_string());
                }else{
                    return EndStatus::NeedMoreLine;
                }
            }else if begin == "{" {
                if feeder.starts_with("}"){
                    if empty {
                        return EndStatus::UnexpectedSymbol("}".to_string());
                    }
                    return EndStatus::NormalEnd;
                }else if feeder.starts_with(")"){
                    return EndStatus::UnexpectedSymbol(")".to_string());
                }else{
                    return EndStatus::NeedMoreLine;
                }
            }else{
                return EndStatus::NormalEnd;
            }
        }

        if feeder.starts_with(")"){
            return EndStatus::UnexpectedSymbol(")".to_string());
        }
        if feeder.starts_with("}"){
            return EndStatus::UnexpectedSymbol("}".to_string());
        }

        return EndStatus::NormalEnd;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if feeder.len() == 0 {
            return None;
        };
    
        let mut ans = Script::new();
        loop{ 
            if Self::eat_job(feeder, core, &mut ans){
                continue;
            }
            ans.text += &feeder.consume_blank_return();

            match Self::check_end(feeder, core, ans.list.len() == 0) {
                EndStatus::UnexpectedSymbol(s) => {
                    eprintln!("Unexpected symbol: {}", s);
                    core.set_var("?", "2");
                    feeder.consume(feeder.len());
                    return None;
                },
                EndStatus::NeedMoreLine => {
                    if ! feeder.feed_additional_line(core) {
                        feeder.consume(feeder.len());
                        return None;
                    }
                },
                EndStatus::NormalEnd => {
                    if ans.list.len() == 0 {
                        core.set_var("?", "2");
                        feeder.consume(feeder.len());
                        return None;
                    }
                    return Some( ans )
                }
            }
        }
    }
}
