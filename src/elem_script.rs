//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use nix::unistd::Pid;
/*
use crate::abst_script_elem::ScriptElem;
use crate::Command;
use nix::sys::wait::waitpid;
use nix::unistd::{Pid, pipe};
use nix::unistd::read;
use nix::sys::wait::WaitStatus;
*/
use crate::elem_pipeline::Pipeline;
use crate::abst_script_elem::ScriptElem;

/* ( script ) */
pub struct Script {
    pub elems: Vec<Pipeline>,
    pub text: String,
}

impl ScriptElem for Script {

    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        for p in &mut self.elems {
            p.exec(conf);
        }
        None
    }
}

impl Script {
    pub fn new() -> Script{
        Script {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Script> {
        let mut ans = Script::new();

        if let Some(p) = Pipeline::parse(text, conf) {
            //TODO: ans.text = p.text;
            ans.elems.push(p);

            return Some(ans);
        }

        None
    }
}
