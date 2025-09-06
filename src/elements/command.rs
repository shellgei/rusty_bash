//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod function_def;
pub mod simple;
pub mod paren;
pub mod brace;
pub mod r#while;
pub mod r#if;

use crate::{ShellCore, Feeder, proc_ctrl, Script};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use self::simple::SimpleCommand;
use self::paren::ParenCommand;
use self::brace::BraceCommand;
use self::function_def::FunctionDefinition;
use self::r#while::WhileCommand;
use self::r#if::IfCommand;
use std::fmt;
use std::fmt::Debug;
use super::{io, Pipe};
use super::io::redirect::Redirect;
use nix::unistd;
use nix::unistd::{ForkResult, Pid};

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

impl Clone for Box::<dyn Command> {
    fn clone(&self) -> Box<dyn Command> {
        self.boxed_clone()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        if self.force_fork() || pipe.is_connected() {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core)
        }
    }

    fn fork_exec_child(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<(), ExecError> {
        core.initialize_as_subshell(Pid::from_raw(0), pipe.pgid);
        io::connect(pipe, self.get_redirects(), core)?;
        self.run(core, true)
    } 

    fn fork_exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        match unsafe{unistd::fork()?} {
            ForkResult::Child => {
                if let Err(e) = self.fork_exec_child(core, pipe) {
                    e.print(core);
                    core.db.set_param("?", "1", None)?;
                }
                exit::normal(core)
            },
            ForkResult::Parent { child } => {
                proc_ctrl::set_pgid(core, child, pipe.pgid);
                pipe.parent_close();
                Ok(Some(child))
            },
        }
    }

    fn nofork_exec(&mut self, core: &mut ShellCore) -> Result<Option<Pid>, ExecError> {
        let mut result = Ok(None);
        for r in self.get_redirects().iter_mut() {
            if let Err(e) = r.connect(true, core) {
                result = Err(e);
            }   
        }   

        if result.is_ok() {
            let _ = self.run(core, false);
        }else{
            core.db.set_param("?", "1", None)?;
        }
        self.get_redirects().iter_mut().rev().for_each(|r| r.restore());
        result
    }  

    fn run(&mut self, _: &mut ShellCore, fork: bool) -> Result<(), ExecError>;
    fn get_text(&self) -> String;
    fn get_redirects(&mut self) -> &mut Vec<Redirect>;
    fn set_force_fork(&mut self);
    fn boxed_clone(&self) -> Box<dyn Command>;
    fn force_fork(&self) -> bool;
}

pub fn eat_inner_script(feeder: &mut Feeder, core: &mut ShellCore,
           left: &str, right: Vec<&str>, ans: &mut Option<Script>) -> Result<bool, ParseError> {
   if ! feeder.starts_with(left) {
       return Ok(false);
    }
    feeder.nest.push( (left.to_string(), right.iter().map(|e| e.to_string()).collect()) );
    feeder.consume(left.len());
    let result_script = Script::parse(feeder, core);
    feeder.nest.pop();
    *ans = result_script?;
    Ok(ans.is_some())
}

fn eat_blank_with_comment(feeder: &mut Feeder, core: &mut ShellCore, ans_text: &mut String) -> bool {
    let blank_len = feeder.scanner_blank(core);
    if blank_len == 0 {
        return false;
    }
    *ans_text += &feeder.consume(blank_len);

    let comment_len = feeder.scanner_comment();
    *ans_text += &feeder.consume(comment_len);
    true
}

pub fn eat_blank_lines(feeder: &mut Feeder, core: &mut ShellCore, ans_text: &mut String)
-> Result<(), ParseError> {
    loop {
        eat_blank_with_comment(feeder, core, ans_text);
        if feeder.starts_with("\n") {
            *ans_text += &feeder.consume(1);
            continue;
        }

        if feeder.len() == 0 {
            feeder.feed_additional_line(core)?;
            continue;
        }

        return Ok(());
    }
}

fn eat_redirect(feeder: &mut Feeder, core: &mut ShellCore,
                     ans: &mut Vec<Redirect>, ans_text: &mut String) -> Result<bool, ParseError> {
    if let Some(r) = Redirect::parse(feeder, core)? {
        *ans_text += &r.text.clone();
        ans.push(r);
        Ok(true)
    }else{
        Ok(false)
    }
}

pub fn eat_redirects(feeder: &mut Feeder, core: &mut ShellCore,
                     ans_redirects: &mut Vec<Redirect>, ans_text: &mut String) -> Result<(), ParseError> {
    loop {
        eat_blank_with_comment(feeder, core, ans_text);
        if ! eat_redirect(feeder, core, ans_redirects, ans_text)?{
            break;
        }
    }
    Ok(())
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Box<dyn Command>>, ParseError> {
    if let Some(a) = FunctionDefinition::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = SimpleCommand::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = IfCommand::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = ParenCommand::parse(feeder, core, false)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = BraceCommand::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = WhileCommand::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else{ Ok(None) }
}
