//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod simple;
pub mod paren;

use crate::{ShellCore, Feeder, Script};
use self::simple::SimpleCommand;
use self::paren::ParenCommand;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore);
    fn get_text(&self) -> String;
}

fn parse_nested_script(feeder: &mut Feeder, core: &mut ShellCore, left: &str) -> Option<Script> {
   if ! feeder.starts_with(left) {
       return None;
    }
    core.nest.push(left.to_string());
    feeder.consume(left.len());
    if let Some(mut s) = Script::parse(feeder, core) {
        core.nest.pop();
        s.text += left;
        Some(s)
    }else{
        core.nest.pop();
        None
    }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Command>> {
    if let Some(a) =      ParenCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = SimpleCommand::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
