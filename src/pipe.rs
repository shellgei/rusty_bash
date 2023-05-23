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
        close(self.my_in, "Cannot close in-pipe");
        dup_and_close(self.my_out, 1);
        dup_and_close(self.prev_out, 0);
    }

    pub fn parent_close(&mut self) {
        close(self.my_out, "Cannot close parent pipe out");
        close(self.prev_out,"Cannot close parent prev pipe out");
    }

    pub fn is_connected(&self) -> bool {
        self.my_in != -1 || self.my_out != -1 || self.prev_out != -1
    }
}

fn close(fd: RawFd, err_str: &str){
    if fd >= 0 {
        unistd::close(fd).expect(err_str);
    }
}

fn dup_and_close(from: RawFd, to: RawFd) {
    if from < 0 || to < 0 {
        return;
    }
    close(to,&("Can't close fd: ".to_owned() + &to.to_string()));
    unistd::dup2(from, to).expect("Can't copy file descriptors");
    close(from, &("Can't close fd: ".to_owned() + &from.to_string()));
}
