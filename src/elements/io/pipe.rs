//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};
use nix::unistd::Pid;
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

    pub fn connect_lastpipe(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.lastpipe && self.prev != 0 {
            self.lastpipe_backup = core.fds.backup(0);
            core.fds.replace(self.prev, 0)?;
        }
        Ok(())
    }

    pub fn restore_lastpipe(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.lastpipe && self.lastpipe_backup != -1 {
            core.fds.replace(self.lastpipe_backup, 0)?;
        }
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Pipe> {
        let len = feeder.scanner_pipe(core);

        if len > 0 {
            Some(Self::new(feeder.consume(len)))
        } else {
            None
        }
    }

    pub fn set(&mut self, prev: RawFd, pgid: Pid, core: &mut ShellCore) {
        if self.text != ">()" {
            (self.recv, self.send) = core.fds.pipe();
            self.prev = prev;
        }

        if self.text == ">()" {
            (self.proc_sub_recv, self.proc_sub_send) = core.fds.pipe();
            self.prev = self.proc_sub_recv;
        }

        self.pgid = pgid;
    }

    pub fn connect(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.text == ">()" {
            core.fds.replace(self.proc_sub_send, 0)?;
        }

        core.fds.close(self.recv);
        core.fds.replace(self.send, 1)?;
        core.fds.replace(self.prev, 0)?;

        if self.text == "|&" {
            core.fds.share(1, 2)?;
        }
        Ok(())
    }

    pub fn parent_close(&mut self, core: &mut ShellCore) {
        core.fds.close(self.send);
        core.fds.close(self.prev);
    }

    pub fn is_connected(&self) -> bool {
        if self.lastpipe {
            return false;
        }
        self.recv != -1 || self.send != -1 || self.prev != -1
    }
}
