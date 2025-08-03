//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod pipe;
pub mod redirect;

use crate::elements::io::redirect::Redirect;
use crate::elements::Pipe;
use crate::error::exec::ExecError;
use crate::ShellCore;
use nix::errno::Errno;
use nix::{fcntl, unistd};
use std::os::unix::prelude::RawFd;

pub fn close(fd: RawFd, err_str: &str) {
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
            close(from, &format!("sush(fatal): {from}: cannot be closed"));
            true
        }
        Err(Errno::EBADF) => {
            eprintln!("sush: {to}: Bad file descriptor");
            false
        }
        Err(_) => {
            eprintln!("sush: dup2 Unknown error");
            false
        }
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
    if fcntl::fcntl(from, fcntl::F_GETFD).is_err() {
        return from;
    }
    fcntl::fcntl(from, fcntl::F_DUPFD_CLOEXEC(10)).expect("Can't allocate fd for backup")
}

pub fn connect(
    pipe: &mut Pipe,
    rs: &mut [Redirect],
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    pipe.connect()?;

    for r in rs.iter_mut() {
        r.connect(false, core)?;
    }
    Ok(())
}
