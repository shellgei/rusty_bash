//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::os::unix::prelude::RawFd;
use nix::unistd;

#[derive(Debug)]
pub struct PipeRecipe {
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
}

impl PipeRecipe {
    pub fn connect(&mut self) {
        close(self.recv, "Cannot close in-pipe");
        replace(self.send, 1);
        replace(self.prev, 0);
    }

    pub fn parent_close(&mut self) {
        close(self.send, "Cannot close parent pipe out");
        close(self.prev,"Cannot close parent prev pipe out");
    }
}

fn close(fd: RawFd, err_str: &str){
    if fd >= 0 {
        unistd::close(fd).expect(err_str);
    }
}

fn replace(from: RawFd, to: RawFd) {
    if from < 0 || to < 0 {
        return;
    }
    close(to,&("Can't close fd: ".to_owned() + &to.to_string()));
    unistd::dup2(from, to).expect("Can't copy file descriptors");
    close(from, &("Can't close fd: ".to_owned() + &from.to_string()))
}
