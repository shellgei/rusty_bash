//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::os::unix::prelude::RawFd;

#[derive(Debug)]
pub struct PipeRecipe {
    pub recv: RawFd,
    pub send: RawFd,
    pub prev: RawFd,
}
