//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_script_elem::ScriptElem;
use nix::unistd::{Pid, fork, ForkResult};
use crate::elem_script::Script;
use nix::sys::wait::{WaitStatus, waitpid};

/* ( script ) */
pub struct CompoundParen {
    pub script: Option<Script>,
    text: String,
    pid: Option<Pid>, 
}

impl ScriptElem for CompoundParen {
    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid>{
        if let Some(s) = &mut self.script {
            return s.exec(conf);
        };

        None
        /*
        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    //self.set_child_io();
                    if let Some(s) = &mut self.script {
                        return s.exec(conf);
                    };
                },
                Ok(ForkResult::Parent { child } ) => {
                    if let Some(s) = &self.script {
                    eprintln!("WAIT");
                        self.wait(&s, child, conf);
                    };
                    return None;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }

        None
        */
    }
}

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            pid: None,
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

    fn wait(&self, com: &Script, child: Pid, conf: &mut ShellCore) -> String {
        let mut ans = "".to_string();

        match waitpid(child, None).expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                conf.vars.insert("?".to_string(), status.to_string());
                if status != 0 {
                    eprintln!("Pid: {:?}, Exit with {:?}", pid, status);
                }
            }
            WaitStatus::Signaled(pid, signal, _) => {
                conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                eprintln!("Unknown error")
            }
        };

        if let Some(c) = ans.chars().last() {
            if c == '\n' {
                return ans[0..ans.len()-1].to_string();
            }
        }
        ans
    }

}
