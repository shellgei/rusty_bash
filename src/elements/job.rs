//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::pipeline::Pipeline;
use crate::{Feeder, ShellCore};
use nix::unistd;
use nix::unistd::{ForkResult, Pid};

#[derive(Debug)]
pub struct Job {
    pub pipelines: Vec<Pipeline>,
    pub pipeline_ends: Vec<String>,
    pub text: String,
}

impl Job {
    pub fn exec(&mut self, core: &mut ShellCore, bg: bool) {
        if bg {
            self.exec_bg(core, self.pipelines.len());
        }else {
            self.exec_fg(core);
        }
    }

    pub fn exec_fg(&mut self, core: &mut ShellCore) {
        let mut do_next = true;
        for (pipeline, end) in self.pipelines.iter_mut()
                          .zip(self.pipeline_ends.iter()) {
            if do_next {
                let pids = pipeline.exec(core);
                core.wait_pipeline(pids);
            }
            do_next = (&core.vars["?"] == "0") == (end == "&&");
        }
    }

    fn exec_bg(&mut self, core: &mut ShellCore, pipeline_num: usize) {
        if pipeline_num == 0 {
            panic!("SUSH INTERNAL ERROR (no pipeline)");
        }else if pipeline_num == 1 {
            self.pipelines[0].exec(core);
        }else{
            self.fork_exec(core);
        }
    }

    fn fork_exec(&mut self, core: &mut ShellCore) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.set_pgid(Pid::from_raw(0), Pid::from_raw(0));
                core.set_subshell_vars();
                self.exec_fg(core);
                core.exit()
            },
            Ok(ForkResult::Parent { child } ) => {
                core.set_pgid(child, child);
                Some(child) 
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    fn new() -> Job {
        Job {
            text: String::new(),
            pipelines: vec![],
            pipeline_ends: vec![],
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

    fn eat_pipeline(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        match Pipeline::parse(feeder, core){
            Some(pipeline) => {
                ans.text += &pipeline.text.clone();
                ans.pipelines.push(pipeline);
                true
            },
            None => false,
        }
    }

    fn eat_and_or(feeder: &mut Feeder, ans: &mut Job, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_and_or(core);
        let end = feeder.consume(num);
        ans.pipeline_ends.push(end.clone());
        ans.text += &end;
        num != 0
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
