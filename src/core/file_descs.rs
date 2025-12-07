//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use nix::fcntl;
use nix::unistd::Pid;
use std::os::fd::{OwnedFd, FromRawFd, RawFd};
use nix::unistd;
use std::os::fd::AsRawFd;
use nix::errno::Errno;
use std::fs::File;

#[derive(Default, Debug)]
pub struct FileDescriptors {
    fds: Vec<Option<OwnedFd>>,
}

impl FileDescriptors {
    pub(super) fn new() -> Self {
        let mut data = Self::default();
        for _ in 0..256 {
            data.fds.push(None);
        }

        data
    }

    pub(super) fn dupfd_cloexec(&mut self, from: RawFd,
                                hereafter: RawFd) -> Result<RawFd, ExecError> {
        let fd = fcntl::fcntl(from, fcntl::F_DUPFD_CLOEXEC(hereafter))?;
        self.fds[fd as usize] = Some(unsafe { OwnedFd::from_raw_fd(fd) });

        Ok(fd)
    }

    pub fn tcsetpgrp(&mut self, fd: RawFd, pgid: Pid) -> Result<(), ExecError> {
        if let Some(fd) = self.fds[fd as usize].as_mut() {
            return Ok(unistd::tcsetpgrp(fd, pgid)?);
        }
        Ok(())
    }

    pub fn tcgetpgrp(&mut self, fd: RawFd) -> Result<Pid, ExecError> {
        if let Some(fd) = self.fds[fd as usize].as_mut() {
            return Ok(unistd::tcgetpgrp(fd)?);
        }
        Err(ExecError::Other("cannot get process group".to_string()))
    }

    pub fn close(&mut self, fd: RawFd, _: &str) {
        if fd >= 3 {
            if self.fds[fd as usize].is_some() {
                self.fds[fd as usize] = None;
            }
        }

        if fd >= 0 {
            let _ = unistd::close(fd);
        }
    }

    pub fn pipe(&mut self) -> (RawFd, RawFd) {
        let (recv, send) = unistd::pipe().expect("Cannot open pipe");
        let fd_recv = recv.as_raw_fd();
        let fd_send = send.as_raw_fd();

        self.fds[fd_recv as usize] = Some(recv);
        self.fds[fd_send as usize] = Some(send);

        (fd_recv, fd_send)
    }

    pub fn backup(&mut self, from: RawFd) -> RawFd {
        if fcntl::fcntl(from, fcntl::F_GETFD).is_err() {
            return from;
        }
        self.dupfd_cloexec(from, 10).unwrap()
    }

    pub fn replace(&mut self, from: RawFd, to: RawFd) -> bool {
        if from < 0 || to < 0 {
            return false;
        }
    
        match unistd::dup2(from, to) {
            Ok(_) => {
                self.close(from, &format!("sush(fatal): {from}: cannot be closed"));
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

    pub fn share(&mut self, from: RawFd, to: RawFd) -> Result<(), ExecError> {
        if from < 0 || to < 0 {
            return Err(ExecError::Other("minus fd number".to_string()));
        }
    
        match unistd::dup2(from, to) {
            Ok(_) => Ok(()),
            Err(Errno::EBADF) => Err(ExecError::BadFd(to)),
            Err(_) => Err(ExecError::Other("dup2 Unknown error".to_string())),
        }
    }

    pub fn get_file(&mut self, fd: RawFd) -> File {
        let f = self.fds[fd as usize].as_mut().unwrap().try_clone().unwrap();
        self.fds[fd as usize] = None;
        File::from(f)
    }
}
