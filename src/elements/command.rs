//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod simple;
use crate::ShellCore;

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore);
    fn get_text(&self) -> String;
}
