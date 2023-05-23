//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::os::unix::prelude::RawFd;

#[derive(Debug)]
pub struct Pipe {
    pub my_in: RawFd,
    pub my_out: RawFd,
    pub prev_out: RawFd,
}

impl Pipe { pub fn parent_close(&mut self){ } }
