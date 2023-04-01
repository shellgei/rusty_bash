//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::job::Job;

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

    fn unexpected_symbol(feeder: &mut Feeder) -> bool {
        if feeder.len() == 0 {
            return false;
        }

        if let Some(_) =  ")}".find(feeder.nth(0)){
            return true;
        }

        false
    }

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

    fn check_end(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        if let Some(begin) = core.nest.pop() {
            core.nest.push(begin.clone());
            if begin == "(" {
                return feeder.starts_with(")");
            }else if begin == "{" {
                return feeder.starts_with("}");
            }else{
                return true;
            }
        }
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Script> {
        if feeder.len() == 0 {
            return None;
        };
    
        if Self::unexpected_symbol(feeder) {
            eprintln!("Unexpected symbol: {}", feeder.consume(feeder.len()).trim_end());
            core.set_var("?", "2");
            return None;
        }

        let backup = feeder.clone();
        let mut ans = Script::new();
        loop{ 
            if Self::eat_job(feeder, core, &mut ans){
                continue;
            }
            ans.text += &feeder.consume_blank_return();

            if Self::check_end(feeder, core) {
                if ans.list.len() > 0 {
                    return Some( ans )
                }else{
                    //eprintln!("EMPTY");
                    feeder.consume(feeder.len());
                    return None;
                }
            }else{
                if ! feeder.feed_additional_line(core) {
                    feeder.consume(feeder.len());
                    return None;
                }
            }
        }
    }
}
