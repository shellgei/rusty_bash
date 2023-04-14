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

fn eat_script(feeder: &mut Feeder, core: &mut ShellCore, script: &mut Option<Script>, text: &mut String) -> bool {
    if let Some(s) = Script::parse(feeder, core) {
        *text += &s.text;
        *script = Some(s);
        return true;
    }
    false
}

/*
pub fn parse_nested_script(feeder: &mut Feeder, core: &mut ShellCore, text: &mut String) -> Option<Box<dyn Command>> {
    text = feeder.consume(1);

    if ! eat_script(feeder, core, &mut ans){
        core.nest.pop();
        return None;
    }
    ans.text += &feeder.consume(1);

    core.nest.pop();
    Some(ans)
}
*/

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Command>> {
    if let Some(a) =      ParenCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = SimpleCommand::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
