//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::command::{
    BraceCommand, IfCommand, ParenCommand, SimpleCommand, WhileCommand
};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils;
use crate::{Feeder, ShellCore};
use nix::unistd::Pid;
use nix::sys::wait::WaitStatus;
//use std::{thread, time};
use crate::core::jobtable::JobEntry;

#[derive(Debug, Clone, Default)]
pub struct Coprocess {
    pub text: String,
    name: String,
    command: Option<Box<dyn Command>>,
    force_fork: bool,
    _dummy: Vec<Redirect>,
    lineno: usize,
}

impl Command for Coprocess {
    fn exec(&mut self, core: &mut ShellCore, _: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        if core.break_counter > 0 || core.continue_counter > 0 {
            return Ok(None);
        }
        core.jobtable_check_status()?;

        let backup = core.tty_fd.clone();
        core.tty_fd = None;
        let pgid = Pid::from_raw(0);
        let mut com = self.command.clone().unwrap().clone();
        com.set_force_fork();

        let mut prevp = Pipe::new("|".to_string());
        prevp.set(-1, pgid, core);
        prevp.send = core.fds.dupfd_cloexec(prevp.send, 60).unwrap();
        prevp.recv = core.fds.dupfd_cloexec(prevp.recv, 60).unwrap();

        let mut lastp = Pipe::new("|".to_string());
        lastp.set(prevp.recv, pgid, core);
        lastp.send = core.fds.dupfd_cloexec(lastp.send, 60).unwrap();
        lastp.recv = core.fds.dupfd_cloexec(lastp.recv, 60).unwrap();

        let pid = com.exec(core, &mut lastp)?.unwrap();
        let fds = vec![lastp.recv.clone(), prevp.send.clone()];
        let fds_str = Some(vec![lastp.recv.to_string(), prevp.send.to_string()]);

        let _ = core.db.init_array(&self.name, fds_str, Some(0), true);
        let pid_name = format!("{}_PID", &self.name);
        let _ = core.db.set_param(&pid_name, &pid.to_string(), Some(0));

        core.fds.close(lastp.send);
        core.fds.close(prevp.recv);

        let _ = core.db.set_param("!", &pid.to_string(), None);
        let new_job_id = core.generate_new_job_id();

        if core.db.flags.contains('i') {
            eprintln!("[{}] {}", &new_job_id, &pid);
        }
        core.job_table_priority.insert(0, new_job_id);
        let mut entry = JobEntry::new(
            vec![Some(pid)],
            &vec![WaitStatus::StillAlive; 1],
            &self.get_one_line_text(),
            "Running",
            new_job_id,
        );
        entry.coproc_name = Some(self.name.clone());
        entry.coproc_fds = fds;

        if let Some(pid) = core.get_jobentry_pid_by_coproc_name(&self.name) {
            let msg = format!("warning: execute_coproc: coproc [{}:{}] still exists",
                              &pid, &self.name);
            let err = ExecError::Other(msg);
            err.print(core);
        }

        if !core.options.query("monitor") {
            entry.no_control = true;
        }

        core.job_table.push(entry);
        core.tty_fd = backup;

        //thread::sleep(time::Duration::from_millis(100));
        //core.jobtable_check_status()?;
        Ok(None)
    }

    fn run(&mut self, _: &mut ShellCore, _: bool) -> Result<(), ExecError> {
        Ok(())
    }
    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self._dummy
    }
    fn get_lineno(&mut self) -> usize {
        self.lineno
    }
    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
    fn boxed_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn force_fork(&self) -> bool {
        self.force_fork
    }

    fn pretty_print(&mut self, indent_num: usize) {
        self.pretty_print(indent_num);
    }
}

impl Coprocess {
    pub fn pretty_print(&mut self, indent_num: usize) {
        println!("{} () ", self.name);
        for com in self.command.iter_mut() {
            com.pretty_print(indent_num);
        }
    }

    fn eat_header(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        self.text += &feeder.consume(6);
        command::eat_blank_with_comment(feeder, core, &mut self.text);

        let len = feeder.scanner_name(core);
        self.name = feeder.consume(len).to_string();
        self.text += &self.name;

        if utils::reserved(&self.name) {
            return Err(ParseError::UnexpectedSymbol("coproc".to_string()));
        }else if self.name.is_empty() {
            self.name = "COPROC".to_string();
        }

        command::eat_blank_with_comment(feeder, core, &mut self.text);
        Ok(())
    }

    fn eat_body(&mut self, feeder: &mut Feeder, core: &mut ShellCore) -> Result<(), ParseError> {
        self.command = if let Some(a) = IfCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else if let Some(a) = ParenCommand::parse(feeder, core, false)? {
            Some(Box::new(a))
        } else if let Some(a) = BraceCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else if let Some(a) = WhileCommand::parse(feeder, core)? {
            Some(Box::new(a))
        } else {
            None
        };

        if let Some(c) = &self.command {
            self.text += &c.get_text();
        }
        Ok(())
    }

    fn parse_simple_command(feeder: &mut Feeder,
        core: &mut ShellCore
    ) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        ans.text += &feeder.consume(6);
        command::eat_blank_with_comment(feeder, core, &mut ans.text);

        ans.command = if let Some(a) = SimpleCommand::parse(feeder, core)? {
            ans.text += &a.get_text();
            ans.name = "COPROC".to_string();
            Some(a)
        }else{
            return Err(ParseError::UnexpectedSymbol("coproc".to_string()));
        };

        Ok(Some(ans))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("coproc") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.lineno = feeder.lineno;

        feeder.set_backup();

        ans.eat_header(feeder, core)?;
        ans.eat_body(feeder, core)?;

        if ans.command.is_some() {
            feeder.pop_backup();
            Ok(Some(ans))
        } else {
            feeder.rewind();
            Self::parse_simple_command(feeder, core)
        }
    }
}
