//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::os::unix::prelude::RawFd;
use nix::unistd::{close,dup2};

#[derive(Debug)]
pub struct Pipe {
    pub my_in: RawFd,
    pub my_out: RawFd,
    pub prev_out: RawFd,
}

impl Pipe {
    pub fn connect(&mut self) {
        if self.my_in != -1 {
            close(self.my_in).expect("Cannot close in-pipe");
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
    close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    dup2(from, to).expect("Can't copy file descriptors");
    close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}
