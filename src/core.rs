//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod database;
pub mod history;
pub mod jobtable;
mod file_descs;

use crate::file_check;
use crate::core::jobtable::JobEntry;
use crate::elements::substitution::Substitution;
use crate::{proc_ctrl, signal};
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use self::database::DataBase;
use std::{io, env, path, process};
use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use self::file_descs::FileDescriptors;

type BuiltinFn = fn(&mut ShellCore, &[String]) -> i32;
type SubstBuiltinFn = fn(&mut ShellCore, &[String], &mut [Substitution]) -> i32;

#[derive(Default)]
pub struct ShellCore {
    pub flags: String,
    pub db: DataBase,
    rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
    pub builtins: HashMap<String, BuiltinFn>,
    pub subst_builtins: HashMap<String, SubstBuiltinFn>,
    pub sigint: Arc<AtomicBool>,
    pub is_subshell: bool,
    pub source_function_level: i32,
    pub return_flag: bool,
    pub fds: FileDescriptors,
    pub tty_fd: Option<RawFd>,
    pub job_table: Vec<JobEntry>,
    tcwd: Option<path::PathBuf>, // the_current_working_directory
}

/*
fn is_interactive() -> bool {
    match file_check::is_tty(0) {
        Ok(result) => result,
        Err(err) => panic!("{}", err),
    }
}*/

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore {
            db: DataBase::new(),
            fds: FileDescriptors::new(),
            ..Default::default()
        };

        core.init_current_directory();
        core.set_initial_parameters();
        core.set_builtins();

        if file_check::is_tty(0) {
            core.flags += "i";
            core.tty_fd = Some(core.fds.dupfd_cloexec(0, 255).unwrap());
        }

        let home = core.db.get_param("HOME").unwrap_or(String::new()).to_string();
        core.db.set_param("HISTFILE", &(home + "/.sush_history"), None).unwrap();
        core.db.set_param("HISTFILESIZE", "2000", None).unwrap();

        core
    }

    fn set_initial_parameters(&mut self) {
        self.db.set_param("$", &process::id().to_string(), None).unwrap();
        self.db.set_param("BASHPID", &process::id().to_string(), None).unwrap();
        self.db.set_param("BASH_SUBSHELL", "0", None).unwrap();
        self.db.set_param("?", "0", None).unwrap();
        self.db.set_param("PS1", "\\[\\033[01;36m\\]\\b\\[\\033[00m\\]\\[\\033[01;35m\\]\\w\\[\\033[00m\\](debug)🍣 ", None).unwrap();
        self.db.set_param("PS2", "> ", None).unwrap();
        self.db.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None).unwrap();
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.contains(flag)
    }

    pub fn run_builtin(&mut self, args: &mut [String]) -> bool {
        if args.is_empty() {
            panic!("SUSH INTERNAL ERROR (no arg for builtins)");
        }

        if ! self.builtins.contains_key(&args[0]) {
            return false;
        }

        let func = self.builtins[&args[0]];
        let status = func(self, args);
        let _ = self.db.set_param("?", &status.to_string(), None);
        true
    }

    pub fn run_function(&mut self, args: &mut Vec<String>) -> bool {
        match self.db.functions.get_mut(&args[0]) {
            Some(f) => {f.clone().run_as_command(args, self); true},
            None => false,
        }
    }

    fn set_subshell_parameters(&mut self) {
        let pid = nix::unistd::getpid();
        let _ = self.db.set_param("BASHPID", &pid.to_string(), None);
        let _ = match self.db.get_param("BASH_SUBSHELL").unwrap().parse::<usize>() {
            Ok(num) => self.db.set_param("BASH_SUBSHELL", &(num+1).to_string(), None),
            Err(_) =>  self.db.set_param("BASH_SUBSHELL", "0", None),
        };
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        signal::restore(Signal::SIGINT);

        self.is_subshell = true;
        proc_ctrl::set_pgid(self, pid, pgid);
        self.set_subshell_parameters();
        self.job_table.clear();
    }

    pub fn init_current_directory(&mut self) {
        match env::current_dir() {
            Ok(path) => self.tcwd = Some(path),
            Err(err) => eprintln!("pwd: error retrieving current directory: {:?}", err),
        }
    }

    pub fn get_current_directory(&mut self) -> Option<path::PathBuf> {
        if self.tcwd.is_none() {
            self.init_current_directory();
        }
        self.tcwd.clone()
    }


    pub fn set_current_directory(&mut self, path: &path::PathBuf) -> Result<(), io::Error> {
        let res = env::set_current_dir(path);
        if res.is_ok() {
            self.tcwd = Some(path.clone());
        }
        res
    }
}
