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
            let pids = pipeline.exec(core);
            core.wait_pipeline(pids);
        }
    }

    fn new() -> Job {
        Job {
            text: String::new(),
            pipelines: vec![],
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
        loop {
            while Self::eat_blank_line(feeder, &mut ans, core) {}
            if ! Self::eat_pipeline(feeder, &mut ans, core) ||
               ! Self::eat_and_or(feeder, &mut ans, core) {
                break;
            }
        }

        if ans.pipelines.len() > 0 {
            dbg!("{:?}", &ans);
            Some(ans)
        }else{
            None
        }
    }
}
