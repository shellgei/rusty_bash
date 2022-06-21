//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use nix::unistd::{read, Pid};
use nix::sys::wait::{waitpid, WaitStatus};

use crate::abst_arg_elem::ArgElem;
use crate::abst_script_elem::ScriptElem;
use crate::elem_compound_paren::CompoundParen;

pub struct SubArgCommandExp {
    pub text: String,
    pub pos: DebugInfo,
    pub com: CompoundParen, 
}

impl ArgElem for SubArgCommandExp {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<Vec<String>> {
        self.com.exec(conf);
        if let Some(pid) = self.com.get_pid() {
            let ans = self.wait(pid, conf)
                //.replace(" ", "\n")
                .split(" ")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            
           // eprintln!("COMEXPANDRES {:?}", ans);
            return vec!(ans);
        }
        vec!()
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgCommandExp {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubArgCommandExp> {
        if text.len() == 0 || text.nth(0) != '$' {
            return None;
        }
    
        let backup = text.clone();
        text.consume(1);

        if let Some(mut e) = CompoundParen::parse(text, conf){
            e.expansion = true;
            let ans = SubArgCommandExp {
                text: "$".to_owned() + &e.text.clone(),
                pos: DebugInfo::init(text),
                com: e };
    
            return Some(ans);
        }else{
            text.rewind(backup);
            None
        }
    }

    fn wait(&self, child: Pid, conf: &mut ShellCore) -> String {
        let mut ans = "".to_string();

        let mut ch = [0;1000];
        while let Ok(n) = read(self.com.pipein, &mut ch) {
            ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
            if n < 1000 {
                break;
            };
        };

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
