//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::ShellCore;
use nix::unistd::Pid;

pub trait ScriptElem {
    fn exec(&mut self, _conf: &mut ShellCore) -> Option<Pid> { None }
}

