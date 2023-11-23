//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod simple;
pub mod paren;
pub mod brace;
pub mod r#while;

use crate::{ShellCore, Feeder, Script};
use self::simple::SimpleCommand;
use self::paren::ParenCommand;
use self::brace::BraceCommand;
use self::r#while::WhileCommand;
use std::fmt;
use std::fmt::Debug;
use super::Pipe;
use super::io::redirect::Redirect;
use nix::unistd::Pid;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid>;
    fn get_text(&self) -> String;
    fn set_force_fork(&mut self);
}

pub fn eat_blank_with_comment(feeder: &mut Feeder, core: &mut ShellCore, ans_text: &mut String) -> bool {
    let blank_len = feeder.scanner_blank(core);
    if blank_len == 0 {
        return false;
    }
    *ans_text += &feeder.consume(blank_len);

    let comment_len = feeder.scanner_comment();
    *ans_text += &feeder.consume(comment_len);
    true
}

pub fn eat_inner_script(feeder: &mut Feeder, core: &mut ShellCore,
           left: &str, right: Vec<&str>, ans: &mut Option<Script>) -> bool {
   if ! feeder.starts_with(left) {
       return false;
    }
    core.nest.push( (left.to_string(), right.iter().map(|e| e.to_string()).collect()) );
    feeder.consume(left.len());
    *ans = Script::parse(feeder, core);
    core.nest.pop();
    ! ans.is_none()
}

pub fn eat_redirect(feeder: &mut Feeder, core: &mut ShellCore,
                     ans: &mut Vec<Redirect>, ans_text: &mut String) -> bool {
    if let Some(r) = Redirect::parse(feeder, core) {
        *ans_text += &r.text.clone();
        ans.push(r);
        true
    }else{
        false
    }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Command>> {
    if let Some(a) = SimpleCommand::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = ParenCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = BraceCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = WhileCommand::parse(feeder, core) { Some(Box::new(a)) }
    else{ None }
}
