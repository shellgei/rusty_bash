//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elem_command::Command;

pub struct Pipeline {
    pub commands: Vec<Command>,
    pub text: String,
}
