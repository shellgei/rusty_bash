//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid};
use crate::elem_script::Script;


/* ( script ) */
pub struct CompoundParen {
    pub script: Option<Script>,
    text: String,
}

impl ScriptElem for CompoundParen {

    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        if let Some(s) = &mut self.script {
            return s.exec(conf);
        }
        None
    }
}

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<CompoundParen> {
        if text.len() == 0 || text.nth(0) != '(' {
            return None;
        }

        let backup = text.clone();
        text.consume(1);
        let mut ans = CompoundParen::new();

        if let Some(s) = Script::parse(text, conf, true) {
            ans.text = "(".to_owned() + &s.text + ")";
            ans.script = Some(s);
        }

        if text.len() == 0 || text.nth(0) != ')' {
            text.rewind(backup);
            return None;
        }

        text.consume(1);
        Some(ans)
    }
}
