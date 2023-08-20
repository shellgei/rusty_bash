//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod pipe;
pub mod redirect;

use std::os::unix::prelude::RawFd;
use nix::unistd;
use super::io::redirect::Redirect;

fn close(fd: RawFd, err_str: &str){
    if fd >= 0 {
        unistd::close(fd).expect(err_str);
    }
}

fn replace(from: RawFd, to: RawFd) {
    if from < 0 || to < 0 {
        return;
    }
    unistd::dup2(from, to).expect("Can't copy file descriptors");
    close(from, &("Can't close fd: ".to_owned() + &from.to_string()))
}

fn backup(from: RawFd) -> RawFd {
    unistd::dup(from).expect("Can't allocate fd for backup")
}

fn share(from: RawFd, to: RawFd) {
    if from < 0 || to < 0 {
        return;
    }
    unistd::dup2(from, to).expect("Can't copy file descriptors");
}

pub fn connect_redirects_with_backup(rs: &mut Vec<Redirect>){
    rs.iter_mut().for_each(|r| r.connect(true));
}

pub fn restore_redirects(rs: &mut Vec<Redirect>){
    rs.iter_mut().rev().for_each(|r| r.restore());
}
