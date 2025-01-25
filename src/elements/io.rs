//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod pipe;
pub mod redirect;

use std::os::unix::prelude::RawFd;
use nix::{fcntl, unistd};
use crate::{process, ShellCore};
use crate::error::exec::ExecError;
use nix::errno::Errno;
use crate::elements::Pipe;
use crate::elements::io::redirect::Redirect;

pub fn close(fd: RawFd, err_str: &str){
    if fd >= 0 {
        unistd::close(fd).expect(err_str);
    }
}

pub fn replace(from: RawFd, to: RawFd) -> bool {
    if from < 0 || to < 0 {
        return false;
    }

    match unistd::dup2(from, to) {
        Ok(_) => {
            close(from, &format!("sush(fatal): {}: cannot be closed", from));
            true
        },
        Err(Errno::EBADF) => {
            eprintln!("sush: {}: Bad file descriptor", to);
            false
        },
        Err(_) => {
            eprintln!("sush: dup2 Unknown error");
            false
        },
    }
}

fn share(from: RawFd, to: RawFd) -> Result<(), ExecError> {
    if from < 0 || to < 0 {
        return Err(ExecError::Other("minus fd number".to_string()));
    }

    match unistd::dup2(from, to) {
        Ok(_) => Ok(()),
        Err(Errno::EBADF) => Err(ExecError::BadFd(to)),
        Err(_) => Err(ExecError::Other("dup2 Unknown error".to_string())),
    }
}

pub fn backup(from: RawFd) -> RawFd {
    fcntl::fcntl(from, fcntl::F_DUPFD_CLOEXEC(10))
           .expect("Can't allocate fd for backup")
}

pub fn connect(pipe: &mut Pipe, rs: &mut Vec<Redirect>, core: &mut ShellCore) {
    if let Err(e) = pipe.connect() {
        e.print(core);
        process::exit(1);
    }

    for r in rs.iter_mut() {
        if let Err(e) = r.connect(false, core) {
            e.print(core);
            process::exit(1);
        }
    }
}
