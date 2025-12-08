//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use nix::fcntl;
use nix::unistd::Pid;
use std::os::fd::{BorrowedFd, OwnedFd, FromRawFd, RawFd};
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

        data.fds[0] = Some(unsafe{OwnedFd::from_raw_fd(0)});
        data.fds[1] = Some(unsafe{OwnedFd::from_raw_fd(1)});
        data.fds[2] = Some(unsafe{OwnedFd::from_raw_fd(2)});

        data
    }

    pub(super) fn dupfd_cloexec(&mut self, from: RawFd,
                                hereafter: RawFd) -> Result<RawFd, ExecError> {
        let fd = fcntl::fcntl(self.fds[from as usize].as_ref().unwrap(),
            fcntl::F_DUPFD_CLOEXEC(hereafter))?;
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

    pub fn close(&mut self, fd: RawFd) {
        if fd < 0 || fd >= 256 {
            return;
        }
        self.fds[fd as usize] = None;
        let _ = unistd::close(fd);
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
        if fcntl::fcntl(self.fds[from as usize].as_ref().unwrap(),
                        fcntl::F_GETFD).is_err() {
            return from;
        }
        self.dupfd_cloexec(from, 10).unwrap()
    }

    pub fn connect_file(&mut self, from: BorrowedFd, to: RawFd) -> Result<(), ExecError> {
        if /*from < 0 ||*/ to < 0 {
            return Ok(());
        }

        if self.fds[to as usize].is_none() {
            return Ok(());
        }

        /*
        let f= if self.fds[from as usize].is_none() {
            unsafe{OwnedFd::from_raw_fd(from)}
        }else {
            self.fds[from as usize].as_mut().unwrap().try_clone().unwrap()
        };
        self.fds[from as usize] = None;
        */
        unistd::dup2(from, &mut self.fds[to as usize].as_mut().unwrap())?;
        //self.close(from.as_raw_fd());
        Ok(())
    }

    pub fn replace(&mut self, from: RawFd, to: RawFd) -> Result<(), ExecError> {
        if from < 0 || to < 0 {
            return Ok(());
        }

        if self.fds[to as usize].is_none() {
            return Ok(());
        }

        let f= if self.fds[from as usize].is_none() {
            unsafe{OwnedFd::from_raw_fd(from)}
        }else {
            self.fds[from as usize].as_mut().unwrap().try_clone().unwrap()
        };
        self.fds[from as usize] = None;
        unistd::dup2(f, &mut self.fds[to as usize].as_mut().unwrap())?;
        self.close(from);
        Ok(())
    }

    pub fn share(&mut self, from: RawFd, to: RawFd) -> Result<(), ExecError> {
        if from < 0 || to < 0 {
            return Err(ExecError::Other("minus fd number".to_string()));
        }

        if self.fds[from as usize].is_none()
        || self.fds[to as usize].is_none() {
            return Ok(());
        }

        let f = self.fds[from as usize].as_mut().unwrap().try_clone().unwrap();
        self.fds[from as usize] = None;

        if let Err(e) = unistd::dup2(&f, &mut self.fds[to as usize].as_mut().unwrap()) {
            return match e {
                Errno::EBADF => Err(ExecError::BadFd(to)),
                _ => Err(ExecError::Other("dup2 Unknown error".to_string())),
            };
        }

        self.fds[from as usize] = Some(f);
        Ok(())
    }

    pub fn get_file(&mut self, fd: RawFd) -> File {
        let f = self.fds[fd as usize].as_mut().unwrap().try_clone().unwrap();
        self.fds[fd as usize] = None;
        File::from(f)
    }

    /*
    pub fn isatty(&mut self, fd: RawFd) -> Result<bool, ExecError> {
        if fd < 0 || fd >= 256 || self.fds[fd as usize].is_none() {
            return Err(ExecError::BadFd(fd));
        }
        let f = self.fds[fd as usize].as_mut().unwrap().try_clone().unwrap();
        let ans = unistd::isatty(&f)?;
        Ok(ans)
    }*/
}
