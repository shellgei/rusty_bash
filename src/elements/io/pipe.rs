//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::io;
use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};
use nix::unistd;
use nix::unistd::Pid;
use std::os::fd::IntoRawFd;
use std::os::unix::prelude::RawFd;

#[derive(Debug, Clone)]
pub struct Pipe {
    pub text: String,
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
    pub pgid: Pid,
    pub lastpipe: bool,
    pub lastpipe_backup: RawFd,
    pub proc_sub_recv: RawFd,
    pub proc_sub_send: RawFd,
}

impl Pipe {
    pub fn new(text: String) -> Pipe {
        Pipe {
            text: text.clone(),
            recv: -1,
            send: -1,
            prev: -1,
            pgid: Pid::from_raw(0),
            lastpipe: false,
            lastpipe_backup: -1,
            proc_sub_recv: -1,
            proc_sub_send: -1,
        }
    }

    pub fn end(prev: RawFd, pgid: Pid, lastpipe: bool) -> Pipe {
        let mut p = Pipe::new(String::new());
        p.lastpipe = lastpipe;
        p.prev = prev;
        p.pgid = pgid;
        p
    }

    pub fn connect_lastpipe(&mut self) {
        if self.lastpipe && self.prev != 0 {
            self.lastpipe_backup = io::backup(0);
            io::replace(self.prev, 0);
        }
    }

    pub fn restore_lastpipe(&mut self) {
        if self.lastpipe && self.lastpipe_backup != -1 {
            io::replace(self.lastpipe_backup, 0);
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipe> {
        let len = feeder.scanner_pipe(core);

        if len > 0 {
            Some(Self::new(feeder.consume(len)))
        } else {
            None
        }
    }

    pub fn set(&mut self, prev: RawFd, pgid: Pid) {
        if self.text != ">()" {
            let (recv, send) = unistd::pipe().expect("Cannot open pipe");
            self.recv = recv.into_raw_fd();
            self.send = send.into_raw_fd();
            self.prev = prev;
        }

        if self.text == ">()" {
            let (recv, send) = unistd::pipe().expect("Cannot open pipe");
            self.proc_sub_recv = recv.into_raw_fd();
            self.proc_sub_send = send.into_raw_fd();
            self.prev = self.proc_sub_recv;
        }

        self.pgid = pgid;
    }

    pub fn connect(&mut self) -> Result<(), ExecError> {
        if self.text == ">()" {
            io::replace(self.proc_sub_send, 0);
        }

        io::close(self.recv, "Cannot close in-pipe");
        io::replace(self.send, 1);
        io::replace(self.prev, 0);

        if &self.text == &"|&" {
            io::share(1, 2)?;
        }
        Ok(())
    }

    pub fn parent_close(&mut self) {
        io::close(self.send, "Cannot close parent pipe out");
        io::close(self.prev, "Cannot close parent prev pipe out");
    }

    pub fn is_connected(&self) -> bool {
        if self.lastpipe {
            return false;
        }
        self.recv != -1 || self.send != -1 || self.prev != -1
    }
}
