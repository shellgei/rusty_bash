//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_elems::Compound;
use nix::unistd::{Pid, fork, ForkResult};
use std::os::unix::prelude::RawFd;
use crate::elem_script::Script;
use crate::element_list::ControlOperator;
use std::process::exit;
use crate::elem_redirect::Redirect;
use crate::file_descs::*;
use nix::unistd::{close, pipe};
use crate::scanner::*;
use crate::element_list::CompoundType;

pub struct CompoundParen {
    pub script: Option<Script>,
    text: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub substitution: bool,
    fds: FileDescs,
    my_type: CompoundType, 
}

impl Compound for CompoundParen {
    fn exec(&mut self, conf: &mut ShellCore) {
        let p = pipe().expect("Pipe cannot open");

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                   //self.fds.set_child_io(conf);
                    if let Err(s) = self.fds.set_child_io(conf){
                        eprintln!("{}", s);
                        exit(1);
                    }
                    if let Some(s) = &mut self.script {
                        if self.substitution {
                            close(p.0).expect("Can't close a pipe end");
                            FileDescs::dup_and_close(p.1, 1);
                        }
                        s.exec(conf);
                        close(1).expect("Can't close a pipe end");
                        exit(conf.vars["?"].parse::<i32>().unwrap());
                    };
                },
                Ok(ForkResult::Parent { child } ) => {
                    if self.substitution {
                        close(p.1).expect("Can't close a pipe end");
                        self.substitution_text  = conf.read_pipe(p.0, child)
                            .trim_end_matches('\n').to_string();
                    }
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }
}

impl CompoundParen {
    pub fn new() -> CompoundParen{
        CompoundParen {
            script: None,
            pid: None,
            text: "".to_string(),
            substitution_text: "".to_string(),
            substitution: false,
            my_type: CompoundType::Paren, 
            fds: FileDescs::new(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore, substitution: bool) -> Option<CompoundParen> {
        if text.len() == 0 || text.nth(0) != '(' {
            return None;
        }

        //let mut all_backup = text.clone();
        let mut backup = text.clone();
        let mut ans = CompoundParen::new();
        let mut input_success;

        loop{
            text.consume(1);
            if let Some(s) = Script::parse(text, conf, &ans.my_type) {

                ans.text = "(".to_owned() + &s.text;
                let (n, op) = scanner_control_op(text);
                if let Some(p) = op  {
                    if p != ControlOperator::RightParen {
                        text.rewind(backup);
                        return None;
                    }
                }

                ans.text += &text.consume(n);
                ans.script = Some(s);

            //    break;
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
                continue;
            }

            if ans.text.len() != 0 && ! ans.text.ends_with(")") {
                (backup, input_success) = text.rewind_feed_backup(&backup, conf);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
            }else{
                break;
            }
        }

        //text.consume(1);

        /* distinguish from (( )) */
        if ans.text.starts_with("((") && ans.text.ends_with("))") {
            text.rewind(backup);
            return None;
        }

        if substitution {
            return Some(ans);
        }

        loop {
            let d = scanner_blank(text, 0);
            ans.text += &text.consume(d);

            if let Some(r) = Redirect::parse(text, conf){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else{
                break;
            }
        }

        Some(ans)
    }
}
