//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub trait ElemOfCommand {
    fn parse_info(&self) -> Vec<String>;
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn text(&self) -> String { String::new() }
}

