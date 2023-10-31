//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::pipeline::Pipeline;
use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub text: String,
}

impl Job {
    pub fn exec(&mut self, core: &mut ShellCore) {
        for pipeline in self.pipelines.iter_mut() {
            let is_subshell = core.vars["$"] != core.vars["BASHPID"];
            let pids = pipeline.exec(core, is_subshell);
            core.wait_pipeline(pids);
        }
    }

    fn new() -> Job {
        Job {
            text: String::new(),
            pipelines: vec![]
        }
    }

    fn eat_blank_line(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);
        if feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Job> {
        let mut ans = Self::new();
        while Self::eat_blank_line(feeder, &mut ans, core) {}
        if let Some(pipeline) = Pipeline::parse(feeder, core){
            ans.text += &pipeline.text.clone();
            ans.pipelines.push(pipeline);
            while Self::eat_blank_line(feeder, &mut ans, core) {}
            return Some(ans);
        }
        None
    }
}
