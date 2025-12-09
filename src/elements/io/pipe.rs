//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use std::os::fd::IntoRawFd;
use std::os::unix::prelude::RawFd;
use nix::unistd;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct Pipe {
    pub text: String,
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
    pub pgid: Pid,
}

impl Pipe {
    pub fn new(text: String) -> Pipe {
        Pipe {
            text,
            recv: -1,
            send: -1,
            prev: -1,
            pgid: Pid::from_raw(0),
        }
    }

    pub fn end(prev: RawFd, pgid: Pid) -> Pipe {
        let mut dummy = Pipe::new(String::new());
        dummy.prev = prev;
        dummy.pgid = pgid;
        dummy
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipe> {
        let len = feeder.scanner_pipe(core);

        if len > 0 {
            Some(Self::new(feeder.consume(len)))
        }else{
            None
        }
    }

    pub fn set(&mut self, prev: RawFd, pgid: Pid) {
        let (recv, send) = unistd::pipe().expect("Cannot open pipe");
        self.recv = recv.into_raw_fd();
        self.send = send.into_raw_fd();
        self.prev = prev;
        self.pgid = pgid;
    }

    pub fn connect(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        core.fds.close(self.recv);
        core.fds.replace(self.send, 1)?; 
        core.fds.replace(self.prev, 0)?; 

        if &self.text == "|&" {
            core.fds.share(1, 2)?;
        }
        Ok(())
    }

    pub fn parent_close(&mut self, core: &mut ShellCore) {
        core.fds.close(self.send);
        core.fds.close(self.prev);
    }

    pub fn is_connected(&self) -> bool {
        self.recv != -1 || self.send != -1 || self.prev != -1
    }
}
