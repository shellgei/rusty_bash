//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::io;
use std::os::unix::prelude::RawFd;

#[derive(Debug)]
pub struct PipeRecipe {
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
}

impl PipeRecipe {
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
