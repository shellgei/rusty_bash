//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod jobtable;

use std::collections::HashMap;
use std::os::fd::RawFd;
use std::{io, env, path, process};
use nix::{fcntl, unistd};
use nix::sys::{signal, wait};
use nix::sys::signal::{Signal, SigHandler};
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use crate::core::jobtable::JobEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

pub struct ShellCore {
    pub history: Vec<String>,
    pub flags: String,
    pub vars: HashMap<String, String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
    pub nest: Vec<(String, Vec<String>)>,
    pub sigint: Arc<AtomicBool>,
    pub is_subshell: bool,
    pub tty_fd: RawFd,
    pub job_table: Vec<JobEntry>,
    tcwd: Option<path::PathBuf>, // the_current_working_directory
}

fn is_interactive() -> bool {
    match unistd::isatty(0) {
        Ok(result) => result,
        Err(err) => panic!("{}", err),
    }
}

fn ignore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigIgn) }
        .expect("sush(fatal): cannot ignore signal");
}

fn restore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigDfl) }
        .expect("sush(fatal): cannot restore signal");
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            history: Vec::new(),
            flags: String::new(),
            vars: HashMap::new(),
            builtins: HashMap::new(),
            nest: vec![("".to_string(), vec![])],
            sigint: Arc::new(AtomicBool::new(false)),
            is_subshell: false,
            tty_fd: -1,
            job_table: vec![],
            tcwd: None,
        };

        core.init_current_directory();
        core.set_initial_vars();
        core.set_builtins();

        if is_interactive() {
            core.flags += "i";
            core.tty_fd = fcntl::fcntl(2, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
        }

        core
    }

    fn set_initial_vars(&mut self) {
        self.vars.insert("$".to_string(), process::id().to_string());
        self.vars.insert("BASHPID".to_string(), self.vars["$"].clone());
        self.vars.insert("BASH_SUBSHELL".to_string(), "0".to_string());
        self.vars.insert("?".to_string(), "0".to_string());
        self.vars.insert("HOME".to_string(), env::var("HOME").unwrap_or("/".to_string()));
    }

    pub fn has_flag(&self, flag: char) -> bool {
        if let Some(_) = self.flags.find(flag) {
            return true;
        }
        false
    }

    pub fn wait_process(&mut self, child: Pid) {
        let exit_status = match wait::waitpid(child, None) {
            Ok(WaitStatus::Exited(_pid, status)) => {
                status
            },
            Ok(WaitStatus::Signaled(pid, signal, _coredump)) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal);
                128+signal as i32
            },
            Ok(unsupported) => {
                eprintln!("Unsupported: {:?}", unsupported);
                1
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            },
        };

        if exit_status == 130 {
            self.sigint.store(true, Relaxed);
        }
        self.vars.insert("?".to_string(), exit_status.to_string()); //追加
    } 

    fn set_foreground(&self) {
        if self.tty_fd < 0 { // tty_fdが無効なら何もしない
            return;
        }

        ignore_signal(Signal::SIGTTOU); //SIGTTOUを無視
        unistd::tcsetpgrp(self.tty_fd, unistd::getpid())
            .expect("sush(fatal): cannot get the terminal");
        restore_signal(Signal::SIGTTOU); //SIGTTOUを受け付け
    }

    pub fn wait_pipeline(&mut self, pids: Vec<Option<Pid>>) {
        if pids.len() == 1 && pids[0] == None {
            return;
        }

        for pid in pids {
            self.wait_process(pid.expect("SUSHI INTERNAL ERROR (no pid)"));
        }
        self.set_foreground();
    }

    pub fn run_builtin(&mut self, args: &mut Vec<String>) -> bool {
        if args.len() == 0 {
            panic!("SUSH INTERNAL ERROR (no arg for builtins)");
        }

        if ! self.builtins.contains_key(&args[0]) {
            return false;
        }

        let func = self.builtins[&args[0]];
        let status = func(self, args);
        self.vars.insert("?".to_string(), status.to_string());
        true
    }

    pub fn exit(&self) -> ! {
        let exit_status = match self.vars["?"].parse::<i32>() {
            Ok(n)  => n%256,
            Err(_) => {
                eprintln!("sush: exit: {}: numeric argument required", self.vars["?"]);
                2
            },
        };
    
        process::exit(exit_status)
    }

    fn set_subshell_vars(&mut self) {
        let pid = nix::unistd::getpid();
        self.vars.insert("BASHPID".to_string(), pid.to_string());
        match self.vars["BASH_SUBSHELL"].parse::<usize>() {
            Ok(num) => self.vars.insert("BASH_SUBSHELL".to_string(), (num+1).to_string()),
            Err(_) =>  self.vars.insert("BASH_SUBSHELL".to_string(), "0".to_string()),
        };
    }

    pub fn set_pgid(&self, pid: Pid, pgid: Pid) {
        unistd::setpgid(pid, pgid).expect("sush(fatal): cannot set pgid");
        if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
            self.set_foreground();
        }
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        restore_signal(Signal::SIGINT);

        self.is_subshell = true;
        self.set_pgid(pid, pgid);
        self.set_subshell_vars();
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
