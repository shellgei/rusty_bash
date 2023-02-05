//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod simple;

use crate::Feeder;
use crate::ShellCore;
use self::simple::SimpleCommand;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

pub trait Command {
    fn exec(&mut self, conf: &mut ShellCore);
    fn get_text(&self) -> String;
}

pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn Command>> {
    //else if let Some(a) = CommandParen::parse(text, conf, false)       {Some(Box::new(a))}
    if let Some(a) = SimpleCommand::parse(text, conf)             {Some(Box::new(a))}
    else {None}
}
