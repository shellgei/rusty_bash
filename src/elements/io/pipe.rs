//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::io;
use std::os::unix::prelude::RawFd;
use nix::unistd;

#[derive(Debug)]
pub struct Pipe {
    pub text: String,
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
}

impl Pipe {
    pub fn new(text: String) -> Pipe {
        Pipe { text: text, recv: -1, send: -1, prev: -1 }
    }

    pub fn end(prev: RawFd) -> Pipe {
        let mut dummy = Pipe::new(String::new());
        dummy.prev = prev;
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

    pub fn set(&mut self, prev: RawFd) {
        (self.recv, self.send) = unistd::pipe().expect("Cannot open pipe");
        self.prev = prev;
    }

    pub fn connect(&mut self) {
        io::close(self.recv, "Cannot close in-pipe");
        io::replace(self.send, 1);
        io::replace(self.prev, 0);
    }

    pub fn parent_close(&mut self) {
        io::close(self.send, "Cannot close parent pipe out");
        io::close(self.prev,"Cannot close parent prev pipe out");
    }

    pub fn is_connected(&self) -> bool {
        self.recv != -1 || self.send != -1 || self.prev != -1
    }
}
