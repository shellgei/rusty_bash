//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{close, dup2};
use std::os::unix::prelude::RawFd;
use crate::elem_redirect::Redirect;

pub fn dup_and_close(from: RawFd, to: RawFd){
    close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    dup2(from, to).expect("Can't copy file descriptors");
    close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}

pub fn set_redirect_fds(r: &Box<Redirect>){
    if let Ok(num) = r.path[1..].parse::<i32>(){
        dup2(num, r.left_fd).expect("Invalid fd");
    }else{
        panic!("Invalid fd number");
    }
}
