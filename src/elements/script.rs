//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::CommandType;
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
        }
    }

    /*
    pub fn new() -> Script{
        Script {
            list: vec![],
            text: "".to_string(),
        }
    }
    */

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

        if let Some(j) =  Job::parse(text, conf, parent_type) {
            let txt = j.text.clone();
            Some( Script { list: vec!(j), text: txt } )
        }else{
            None
        }
    }
}
