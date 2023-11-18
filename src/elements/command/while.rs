//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use crate::elements::{command, io};
use nix::unistd;
use super::{Command, Pipe, Redirect};
use nix::sys::termios;
use nix::sys::termios::SetArg::TCSAFLUSH;
use nix::sys::termios::SpecialCharacterIndices;
use nix::unistd::{ForkResult, Pid};
use nix::fcntl;

#[derive(Debug)]
pub struct WhileCommand {
    pub text: String,
    pub condition: Option<Script>,
    pub inner: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork || pipe.is_connected() {
            return self.fork_exec(core, pipe);
        }

        if self.redirects.iter_mut().all(|r| r.connect(true)){
            self.nofork_exec(core);
        }else{
            core.vars.insert("?".to_string(), "1".to_string());
        }
        self.redirects.iter_mut().rev().for_each(|r| r.restore());
        None
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl WhileCommand {
    fn nofork_exec(&mut self, core: &mut ShellCore) {
        let mut ch = [0;16];
        if core.tty_fd >= 0 && ! core.is_subshell {
            core.in_loop = true;
            fcntl::fcntl(core.tty_fd, nix::fcntl::F_SETFL(nix::fcntl::OFlag::O_NDELAY))
                .expect("Can't set nonblock");

            let mut term = termios::tcgetattr(core.tty_fd).expect("!");
            term.control_chars[SpecialCharacterIndices::VINTR as usize] = termios::_POSIX_VDISABLE;
            term.control_chars[SpecialCharacterIndices::VEOF as usize ] = 3; 
            termios::tcsetattr(core.tty_fd, TCSAFLUSH, &term).expect("!");
        }



        loop {
            if core.input_interrupt {
                core.input_interrupt = false;
                break;
            }
            if core.tty_fd >= 0 && ! core.is_subshell {
                if let Ok(n) = unistd::read(core.tty_fd, &mut ch) {
                    let s = String::from_utf8(ch[..n].to_vec()).unwrap();
                    if s == "" { //STOP with CTRL+D
                        break;
                    }
                }
            }

            self.condition.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core, &mut vec![]);

            if core.vars["?"] != "0" {
                break;
            }

            self.inner.as_mut()
                .expect("SUSH INTERNAL ERROR (no script)")
                .exec(core, &mut vec![]);
        }

        if core.tty_fd >= 0 && ! core.is_subshell {
            let mut term = termios::tcgetattr(core.tty_fd).expect("!");
            term.control_chars[SpecialCharacterIndices::VEOF as usize ] = 4; 
            term.control_chars[SpecialCharacterIndices::VINTR as usize] = 3;
            termios::tcsetattr(core.tty_fd, TCSAFLUSH, &term).expect("!");

            fcntl::fcntl(core.tty_fd, nix::fcntl::F_SETFL(nix::fcntl::OFlag::O_SYNC))
                .expect("Can't return from nonblock");
            core.in_loop = false;
        }
    }

    fn fork_exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.initialize_as_subshell(Pid::from_raw(0), pipe.pgid);
                io::connect(pipe, &mut self.redirects);
                self.nofork_exec(core);
                core.exit()
            },
            Ok(ForkResult::Parent { child } ) => {
                core.set_pgid(child, pipe.pgid);
                Some(child) 
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    fn new() -> WhileCommand {
        WhileCommand {
            text: String::new(),
            condition: None,
            inner: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<WhileCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "while", vec!["do"], &mut ans.condition)
        && command::eat_inner_script(feeder, core, "do", vec!["done"],  &mut ans.inner) {
            ans.text.push_str("while");
            ans.text.push_str(&ans.condition.as_mut().unwrap().text.clone());
            ans.text.push_str("do");
            ans.text.push_str(&ans.inner.as_mut().unwrap().text.clone());
            ans.text.push_str(&feeder.consume(4)); //done

            loop {
                command::eat_blank_with_comment(feeder, core, &mut ans.text);
                if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text){
                    break;
                }
            }

            Some(ans)
        }else{
            None
        }
    }
}
