//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use nix::fcntl;
use nix::unistd::Pid;
use std::os::fd::{OwnedFd, FromRawFd, RawFd};
use nix::unistd;

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

}
