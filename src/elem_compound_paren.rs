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

    fn exec(&mut self, _conf: &mut ShellCore) -> Option<Pid>{
 //       self.script.exec()
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
        let mut ans = CompoundParen::new();

        if let Some(s) = Script::parse(text, conf) {
            ans.text = "(".to_owned() + &s.text + ")";
            ans.script = Some(s);
        }

        if text.len() == 0 || text.nth(0) != ')' {
            text.rewind(backup);
            return None;
        }

        Some(ans)
    }
}
