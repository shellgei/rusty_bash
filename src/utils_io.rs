//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{close, dup2, read};
use std::os::unix::prelude::RawFd;
use crate::elem_redirect::Redirect;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;
use nix::unistd::Pid;
use nix::sys::wait::WaitPidFlag;
use nix::sys::wait::{waitpid, WaitStatus};

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

pub fn set_redirect(r: &Box<Redirect>){
    if r.path.len() == 0 {
        panic!("Invalid redirect");
    }

    if r.direction_str == ">" {
        if r.path.chars().nth(0) == Some('&') {
            set_redirect_fds(r);
            return;
        }

        if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
            dup_and_close(file.into_raw_fd(), r.left_fd);
        }else{
            panic!("Cannot open the file: {}", r.path);
        };
    }else if r.direction_str == "&>" {
        if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
            dup_and_close(file.into_raw_fd(), 1);
            dup2(1, 2).expect("Redirection error on &>");
        }else{
            panic!("Cannot open the file: {}", r.path);
        };
    }else if r.direction_str == "<" {
        if let Ok(file) = OpenOptions::new().read(true).open(&r.path){
            dup_and_close(file.into_raw_fd(), r.left_fd);
        }else{
            panic!("Cannot open the file: {}", r.path);
        };
    }
}

pub fn set_child_io(pin: RawFd, pout: RawFd, previn: RawFd, redirects: &Vec<Box<Redirect>>) {

    if pin != -1 {
        close(pin).expect("Cannot close in-pipe");
    }
    if pout != -1 {
        dup_and_close(pout, 1);
    }

    if previn != -1 {
        dup_and_close(previn, 0);
    }

    for r in redirects {
        set_redirect(r);
    };

}

pub fn set_parent_io(pout: RawFd) {
    if pout >= 0 {
        close(pout).expect("Cannot close outfd");
    };
}

pub fn read_pipe(pin: RawFd, pid: Pid) -> String {
    let mut ans = "".to_string();
    let mut ch = [0;1000];

    /*
    if pid == Pid::this() {
        while let Ok(n) = read(pin, &mut ch) {
            ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
            if n < 1000 {
                return ans;
            };
        }
    }*/

    loop {
        while let Ok(n) = read(pin, &mut ch) {
            ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
            match waitpid(pid, Some(WaitPidFlag::WNOHANG)).expect("Faild to wait child process.") {
                WaitStatus::StillAlive => {
                    continue;
                },
                _ => {
                    return ans;
                },
            };
        }
    }
}
