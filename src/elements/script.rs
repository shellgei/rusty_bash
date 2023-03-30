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

    fn unexpected_symbol(feeder: &mut Feeder) -> bool {
        if feeder.len() == 0 {
            return false;
        }

        if let Some(_) =  ")}".find(feeder.nth(0)){
            return true;
        }

        false
    }

    pub fn parse(feeder: &mut Feeder, conf: &mut ShellCore) -> Option<Script> {
        if feeder.len() == 0 {
            return None;
        };
    
        //if feeder.starts_with(")") {
        if Self::unexpected_symbol(feeder) {
            eprintln!("Unexpected symbol: {}", feeder.consume(feeder.len()).trim_end());
            conf.set_var("?", "2");
            return None;
        }

        let backup = feeder.clone();
        let mut ans = Script::new();
        loop {
            ans.text += &feeder.consume_blank();
            if let Some(j) =  Job::parse(feeder, conf) {
                ans.text += &j.text.clone();
                ans.list.push(j);
            }else{
                break;
            }
        }

        if ans.list.len() > 0 {
            Some( ans )
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
