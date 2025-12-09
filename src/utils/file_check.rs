//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

extern crate libc;
use libc::isatty;
use std::os::fd::RawFd;
use std::path::Path;

pub fn is_regular_file(name: &str) -> bool {
    Path::new(name).is_file()
}

pub fn is_tty(fd: RawFd) -> bool {
    unsafe{isatty(fd) == 1}
}

/*
pub fn is_tty_str(name: &str) -> bool {
    let fd = match name.parse::<i32>() {
        Ok(n) => n,
        _ => return false,
    };
    is_tty(fd)
}*/
