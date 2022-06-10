//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_hand_input_unit::HandInputUnit;
use crate::Command;
use crate::elem_arg_delimiter::ArgDelimiter;
use nix::sys::wait::waitpid;
use nix::unistd::{Pid, pipe};
use nix::unistd::read;
use nix::sys::wait::WaitStatus;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Pipeline {
    pub commands: Vec<Command>,
    text: String,
    pub expansion: bool,
}

impl HandInputUnit for Pipeline {

    fn exec(&mut self, conf: &mut ShellCore) -> (Option<Pid>, String){
        if self.expansion {
            self.set_command_expansion_pipe();
        }

        for c in self.commands.iter_mut().rev() {
            let (pid, _) = c.exec(conf);
            c.pid = pid;
        }

        let mut s = "".to_string();
        for c in &self.commands {
            if let Some(p) = c.pid {
                s += &self.wait_command(&c, p, conf);
            };
        }
        (None, s)

        /*
        let x = self.commands.len();
        if x == 0 {
            return (None, "".to_string());
        }

        let (pid_opt, _) = self.commands[x-1].exec(conf);

        if let Some(pid) = pid_opt {
            let result_string = self.wait_command(&self.commands[x-1], pid, conf);
            (None, result_string)
        }else{
            (None, "".to_string())
        }
        */
    }
}

impl Pipeline {
    pub fn new() -> Pipeline{
        Pipeline {
            commands: vec!(),
            expansion: false,
            text: "".to_string(),
        }
    }

    fn set_command_expansion_pipe(&mut self){
        let x = self.commands.len();
        let c = &mut self.commands[x-1];
        let p = pipe().expect("Pipe cannot open");
        c.infd_expansion = p.0;
        c.outfd_expansion = p.1;
        c.expansion = true;
    }

    fn wait_command(&self, com: &Command, child: Pid, conf: &mut ShellCore) -> String {
        let mut ans = "".to_string();

        if com.expansion {
            let mut ch = [0;1000];
            while let Ok(n) = read(com.infd_expansion, &mut ch) {
                ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
                if n < 1000 {
                    break;
                };
            };
        }

        match waitpid(child, None)
            .expect("Faild to wait child process.") {
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

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Pipeline> {
        let mut ans = Pipeline::new();

        loop {
            if let Some(c) = Command::parse(text, conf) {
                let mut eoc = "".to_string();
                if let Some(e) = c.args.last() {
                    eoc = e.text();
                }

                ans.text += &c.text.clone();
                ans.commands.push(c);

                if eoc != "|" {
                    break;
                }

                if let Some(d) = ArgDelimiter::parse(text) {
                    ans.text += &d.text.clone();
                }

                /*

                if text.len() == 0{
                    break;
                }

                if text.nth(0) != '|' {
                    break;
                }
                */

            }else{
                break;
            }
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
