//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod arithmetic;
pub mod brace;
pub mod case;
pub mod r#for;
pub mod function_def;
pub mod r#if;
pub mod paren;
pub mod repeat;
pub mod simple;
pub mod test;
pub mod r#while;

use self::arithmetic::ArithmeticCommand;
use self::brace::BraceCommand;
use self::case::CaseCommand;
use self::function_def::FunctionDefinition;
use self::paren::ParenCommand;
use self::r#for::ForCommand;
use self::r#if::IfCommand;
use self::r#while::WhileCommand;
use self::repeat::RepeatCommand;
use self::simple::SimpleCommand;
use self::test::TestCommand;
use super::io::redirect::Redirect;
use super::{io, Pipe};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::exit;
use crate::{proc_ctrl, Feeder, Script, ShellCore};
use nix::unistd;
use nix::unistd::{ForkResult, Pid};
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Box<dyn Command> {
        self.boxed_clone()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        core.db
            .set_param("LINENO", &self.get_lineno().to_string(), None)?;
        if self.force_fork() || (!pipe.lastpipe && pipe.is_connected()) {
            self.fork_exec(core, pipe)
        } else {
            pipe.connect_lastpipe();
            self.nofork_exec(core)
        }
    }

    fn fork_exec_child(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<(), ExecError> {
        core.initialize_as_subshell(Pid::from_raw(0), pipe.pgid);
        io::connect(pipe, self.get_redirects(), core)?;
        self.run(core, true)
    }

    fn fork_exec(
        &mut self,
        core: &mut ShellCore,
        pipe: &mut Pipe,
    ) -> Result<Option<Pid>, ExecError> {
        match unsafe { unistd::fork()? } {
            ForkResult::Child => {
                if let Err(e) = self.fork_exec_child(core, pipe) {
                    e.print(core);
                    core.db.exit_status = 1;
                }
                exit::normal(core)
            }
            ForkResult::Parent { child } => {
                proc_ctrl::set_pgid(core, child, pipe.pgid);
                pipe.parent_close();
                Ok(Some(child))
            }
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
        } else {
            core.db.exit_status = 1;
        }
        self.get_redirects()
            .iter_mut()
            .rev()
            .for_each(|r| r.restore());
        result
    }

    fn run(&mut self, _: &mut ShellCore, fork: bool) -> Result<(), ExecError>;
    fn get_text(&self) -> String;
    fn get_one_line_text(&self) -> String {
        self.get_text().replace("\n", " ")
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect>;
    fn get_lineno(&mut self) -> usize {
        panic!("IMPLEMENT!!")
    }
    fn set_force_fork(&mut self);
    fn boxed_clone(&self) -> Box<dyn Command>;
    fn force_fork(&self) -> bool;

    fn read_heredoc(
        &mut self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<(), ParseError> {
        for r in self.get_redirects().iter_mut() {
            if r.called_as_heredoc {
                continue;
            }
            if r.symbol == "<<" || r.symbol == "<<-" {
                r.called_as_heredoc = true;
                r.eat_heredoc(feeder, core)?;
            }
        }
        Ok(())
    }

    fn pretty_print(&mut self, indent_num: usize) {
        for _ in 0..indent_num {
            print!("    ");
        }
        println!("{}", &self.get_text());
    }
}

pub fn eat_inner_script(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    left: &str,
    right: Vec<&str>,
    ans: &mut Option<Script>,
    permit_empty: bool,
) -> Result<bool, ParseError> {
    if !feeder.starts_with(left) {
        return Ok(false);
    }
    feeder.nest.push((
        left.to_string(),
        right.iter().map(|e| e.to_string()).collect(),
    ));
    feeder.consume(left.len());
    let result_script = Script::parse(feeder, core, permit_empty);
    feeder.nest.pop();
    *ans = result_script?;
    Ok(ans.is_some())
}

pub fn eat_blank_with_comment(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    ans_text: &mut String,
) -> bool {
    let blank_len = feeder.scanner_blank(core);
    *ans_text += &feeder.consume(blank_len);

    let comment_len = feeder.scanner_comment();
    if comment_len + blank_len == 0 {
        return false;
    }
    *ans_text += &feeder.consume(comment_len);
    true
}

pub fn eat_blank_lines(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    ans_text: &mut String,
) -> Result<(), ParseError> {
    loop {
        eat_blank_with_comment(feeder, core, ans_text);
        if feeder.starts_with("\n") {
            *ans_text += &feeder.consume(1);
            continue;
        }

        if feeder.is_empty() {
            feeder.feed_additional_line(core)?;
            continue;
        }

        return Ok(());
    }
}

fn eat_redirect(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    ans: &mut Vec<Redirect>,
    ans_text: &mut String,
) -> bool {
    if let Some(r) = Redirect::parse(feeder, core) {
        *ans_text += &r.text.clone();
        ans.push(r);
        true
    } else {
        false
    }
}

pub fn eat_redirects(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    ans_redirects: &mut Vec<Redirect>,
    ans_text: &mut String,
) -> Result<bool, ParseError> {
    let mut exist = false;
    loop {
        eat_blank_with_comment(feeder, core, ans_text);
        if !eat_redirect(feeder, core, ans_redirects, ans_text) {
            break;
        } else {
            exist = true;
        }
    }

    Ok(exist)
}

pub fn parse(
    feeder: &mut Feeder,
    core: &mut ShellCore,
) -> Result<Option<Box<dyn Command>>, ParseError> {
    if let Some(a) = FunctionDefinition::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = SimpleCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = IfCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = ArithmeticCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = ParenCommand::parse(feeder, core, false)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = BraceCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = ForCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = WhileCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = RepeatCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = CaseCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = TestCommand::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else {
        Ok(None)
    }
}
