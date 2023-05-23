//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::os::unix::prelude::RawFd;
use nix::unistd;

#[derive(Debug)]
pub struct Pipe {
    pub my_in: RawFd,
    pub my_out: RawFd,
    pub prev_out: RawFd,
}

impl Pipe {
    pub fn connect(&mut self) {
        if self.my_in != -1 {
            unistd::close(self.my_in).expect("Cannot close in-pipe");
        }
        if self.my_out != -1 {
            dup_and_close(self.my_out, 1);
        }

        if self.prev_out != -1 {
            dup_and_close(self.prev_out, 0);
        }
    }

    pub fn is_connected(&self) -> bool {
        self.my_in != -1 || self.my_out != -1 || self.prev_out != -1
    }
}

pub fn dup_and_close(from: RawFd, to: RawFd){
    unistd::close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    unistd::dup2(from, to).expect("Can't copy file descriptors");
    unistd::close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}
