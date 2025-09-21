//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::substitution::Substitution;

pub fn local(core: &mut ShellCore, args: &[String],
             subs: &mut [Substitution]) -> i32 {
    dbg!("{:?}", &subs);
    0
}
