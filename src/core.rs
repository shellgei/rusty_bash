//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod data;
pub mod history;
pub mod jobtable;
pub mod options;

use self::data::Data;
use self::options::Options;
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::{io, env, path, process};
use nix::{fcntl, unistd};
use nix::sys::{resource, signal, wait};
use nix::sys::resource::UsageWho;
use nix::sys::signal::{Signal, SigHandler};
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::sys::time::{TimeSpec, TimeVal};
use nix::time;
use nix::time::ClockId;
use nix::unistd::Pid;
use crate::utils::{error, exit, random, clock};
use crate::core::jobtable::JobEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

pub struct MeasuredTime {
    pub real: TimeSpec, 
    pub user: TimeVal, 
    pub sys: TimeVal, 
}

impl Default for MeasuredTime {
    fn default() -> Self {
        Self {
            real: TimeSpec::new(0,0),
            user: TimeVal::new(0,0),
            sys: TimeVal::new(0,0),
        }
    }
}

#[derive(Default)]
pub struct ShellCore {
    pub data: Data,
    rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
    pub sigint: Arc<AtomicBool>,
    pub read_stdin: bool,
    pub word_eval_error: bool,
    pub is_subshell: bool,
    pub source_function_level: i32,
    pub source_level: i32,
    pub eval_level: i32,
    pub loop_level: i32,
    pub break_counter: i32,
    pub return_flag: bool,
    pub tty_fd: Option<OwnedFd>,
    pub job_table: Vec<JobEntry>,
    pub job_table_priority: Vec<usize>,
    current_dir: Option<path::PathBuf>, // the_current_working_directory
    pub completion_functions: HashMap<String, String>,
    pub completion_actions: HashMap<String, String>,
    pub measured_time: MeasuredTime,
    pub options: Options,
    pub shopts: Options,
    pub suspend_e_option: bool,
    pub script_name: String,
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
            data: Data::new(),
            sigint: Arc::new(AtomicBool::new(false)),
            read_stdin: true,
            options: Options::new_as_basic_opts(),
            shopts: Options::new_as_shopts(),
            script_name: "-".to_string(),
            ..Default::default()
        };

        core.init_current_directory();
        core.set_initial_parameters();
        core.set_builtins();
        ignore_signal(Signal::SIGPIPE);
        ignore_signal(Signal::SIGTSTP);

        core.data.set_param("PS4", "+ ");

        if unistd::isatty(0) == Ok(true) {
            core.data.flags += "i";
            core.read_stdin = false;
            core.data.set_param("PS1", "🍣 ");
            core.data.set_param("PS2", "> ");
            let fd = fcntl::fcntl(0, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            core.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }

        let home = core.data.get_param("HOME").to_string();
        core.data.set_param("HISTFILE", &(home + "/.sush_history"));
        core.data.set_param("HISTFILESIZE", "2000");

        core
    }

    fn set_initial_parameters(&mut self) {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let profile = env!("CARGO_BUILD_PROFILE").to_string();
        let t_arch = env!("CARGO_CFG_TARGET_ARCH").to_string();
        let t_vendor = env!("CARGO_CFG_TARGET_VENDOR").to_string();
        let t_os = env!("CARGO_CFG_TARGET_OS").to_string();
        let machtype = format!("{}-{}-{}", t_arch, t_vendor, t_os);
        let symbol = "rusty_bash".to_string();
        let vparts: Vec<&str> = version.split('.').collect();
        let versinfo = vec![vparts[0].to_string(), vparts[1].to_string(), vparts[2].to_string(),
                             symbol.clone(), profile.clone(), machtype.clone()];

        self.data.set_param("$", &process::id().to_string());
        self.data.set_param("BASHPID", &process::id().to_string());
        self.data.set_param("BASH_SUBSHELL", "0");
        self.data.set_param("BASH_VERSION", &format!("{}({})-{}", version, symbol, profile));
        self.data.set_param("MACHTYPE", &machtype);
        self.data.set_param("HOSTTYPE", &t_arch);
        self.data.set_param("OSTYPE", &t_os);
        self.data.set_array("BASH_VERSINFO", &versinfo);
        self.data.set_param("?", "0");
        self.data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()));
        self.data.set_special_param("SRANDOM", random::get_srandom, Some(Data::not_set));
        self.data.set_special_param("RANDOM", random::get_random, None);
        self.data.set_special_param("EPOCHSECONDS", clock::get_epochseconds, Some(Data::not_set));
        self.data.set_special_param("EPOCHREALTIME", clock::get_epochrealtime, Some(Data::not_set));
        self.data.set_special_param("SECONDS", clock::get_seconds, Some(clock::set_seconds));
        self.data.set_param("SECONDS", "0");
    }

    pub fn wait_process(&mut self, child: Pid) -> WaitStatus {
        let waitflags = match self.is_subshell {
            true  => None,
            false => Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)
        };

        let ws = wait::waitpid(child, waitflags);

        let exit_status = match ws {
            Ok(WaitStatus::Exited(_pid, status)) => status,
            Ok(WaitStatus::Signaled(pid, signal, coredump)) => error::signaled(pid, signal, coredump),
            Ok(WaitStatus::Stopped(pid, signal)) => {
                eprintln!("Stopped Pid: {:?}, Signal: {:?}", pid, signal);
                148
            },
            Ok(unsupported) => {
                let msg = format!("Unsupported wait status: {:?}", unsupported);
                error::print(&msg, self);
                1
            },
            Err(err) => {
                let msg = format!("Error: {:?}", err);
                exit::internal(&msg);
            },
        };

        if exit_status == 130 {
            self.sigint.store(true, Relaxed);
        }
        self.data.set_layer_param("?", &exit_status.to_string(), 0); //追加
        ws.expect("SUSH INTERNAL ERROR: no wait status")
    }

    fn set_foreground(&self) {
        let fd = match self.tty_fd.as_ref() {
            Some(fd) => fd,
            _        => return,
        };

        let pgid = unistd::getpgid(Some(Pid::from_raw(0)))
                   .expect(&error::internal("cannot get pgid"));

        if unistd::tcgetpgrp(fd) == Ok(pgid) {
            return;
        }

        ignore_signal(Signal::SIGTTOU); //SIGTTOUを無視
        unistd::tcsetpgrp(fd, pgid)
            .expect(&error::internal("cannot get the terminal"));
        restore_signal(Signal::SIGTTOU); //SIGTTOUを受け付け
    }

    fn flip_exit_status(&mut self) {
        match self.data.get_param("?").as_ref() {
            "0" => self.data.set_param("?", "1"),
            _   => self.data.set_param("?", "0"),
        };
    }

    fn show_time(&self) {
            let real_end_time = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();

            let self_usage = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
            let children_usage = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

            let real_diff = real_end_time - self.measured_time.real;
            eprintln!("\nreal\t{}m{}.{:06}s", real_diff.tv_sec()/60,
                      real_diff.tv_sec()%60, real_diff.tv_nsec()/1000);
            let user_diff = self_usage.user_time() + children_usage.user_time() - self.measured_time.user;
            eprintln!("user\t{}m{}.{:06}s", user_diff.tv_sec()/60,
                      user_diff.tv_sec()%60, user_diff.tv_usec());
            let sys_diff = self_usage.system_time() + children_usage.system_time() - self.measured_time.sys;
            eprintln!("sys \t{}m{}.{:06}s", sys_diff.tv_sec()/60,
                      sys_diff.tv_sec()%60, sys_diff.tv_usec());
    }

    fn check_e_option(&mut self) {
        if self.data.get_param("?") != "0" 
        && self.data.flags.contains("e") 
        && ! self.suspend_e_option {
            exit::normal(self);
        }
    }

    pub fn wait_pipeline(&mut self, pids: Vec<Option<Pid>>,
                         exclamation: bool, time: bool) -> Vec<WaitStatus> {
        if pids.len() == 1 && pids[0] == None {
            if time {
                self.show_time();
            }
            if exclamation {
                self.flip_exit_status();
            }
            self.check_e_option();
            return vec![];
        }

        let mut pipestatus = vec![];
        let mut ans = vec![];
        for pid in &pids {
            let ws = self.wait_process(pid.expect("SUSHI INTERNAL ERROR (no pid)"));
            ans.push(ws);

            pipestatus.push(self.data.get_param("?"));
        }

        if time {
            self.show_time();
        }
        self.set_foreground();
        self.data.set_layer_array("PIPESTATUS", &pipestatus, 0);

        if self.options.query("pipefail") {
            pipestatus.retain(|e| e != "0");

            if pipestatus.len() != 0 {
                self.data.set_param("?", &pipestatus.last().unwrap());
            }
        }

        if exclamation {
            self.flip_exit_status();
        }

        self.check_e_option();

        ans
    }

    pub fn run_builtin(&mut self, args: &mut Vec<String>, special_args: &mut Vec<String>) -> bool {
        if args.len() == 0 {
            exit::internal(" (no arg for builtins)");
        }

        if self.builtins.contains_key(&args[0]) {
            let func = self.builtins[&args[0]];
            args.append(special_args);
            let status = func(self, args);
            self.data.set_layer_param("?", &status.to_string(), 0);
            return true;
        }

        false
    }

    fn set_subshell_parameters(&mut self) {
        let pid = nix::unistd::getpid();
        self.data.set_layer_param("BASHPID", &pid.to_string(), 0);
        match self.data.get_param("BASH_SUBSHELL").parse::<usize>() {
            Ok(num) => self.data.set_layer_param("BASH_SUBSHELL", &(num+1).to_string(), 0),
            Err(_) =>  self.data.set_layer_param("BASH_SUBSHELL", "0", 0),
        };
    }

    pub fn set_pgid(&self, pid: Pid, pgid: Pid) {
        let _ = unistd::setpgid(pid, pgid);
        if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
            self.set_foreground();
        }
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        restore_signal(Signal::SIGINT);
        restore_signal(Signal::SIGTSTP);
        restore_signal(Signal::SIGPIPE);

        self.is_subshell = true;
        self.set_pgid(pid, pgid);
        self.set_subshell_parameters();
        self.job_table.clear();
    }

    pub fn init_current_directory(&mut self) {
        match env::current_dir() {
            Ok(path) => self.current_dir = Some(path),
            Err(err) => {
                let msg = format!("pwd: error retrieving current directory: {:?}", err);
                error::print(&msg, self);
            },
        }
    }

    pub fn get_current_directory(&mut self) -> Option<path::PathBuf> {
        if self.current_dir.is_none() {
            self.init_current_directory();
        }
        self.current_dir.clone()
    }


    pub fn set_current_directory(&mut self, path: &path::PathBuf) -> Result<(), io::Error> {
        let res = env::set_current_dir(path);
        if res.is_ok() {
            self.current_dir = Some(path.clone());
        }
        res
    }

    pub fn get_ps4(&mut self) -> String {
        let ps4 = self.data.get_param("PS4").trim_end().to_string();
        let mut multi_ps4 = ps4.to_string();
        for _ in 0..(self.source_level + self.eval_level) {
            multi_ps4 += &ps4;
        }

        multi_ps4
    }
}
