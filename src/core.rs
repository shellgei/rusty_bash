//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod completion;
pub mod database;
pub mod history;
pub mod jobtable;
pub mod options;

use crate::{error, proc_ctrl, signal};
use crate::error::exec::ExecError;
use crate::elements::substitution::Substitution;
use self::database::DataBase;
use self::options::Options;
use self::completion::{Completion, CompletionEntry};
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::{io, env, path};
use nix::{fcntl, unistd};
use nix::sys::signal::Signal;
use nix::sys::time::{TimeSpec, TimeVal};
use nix::unistd::Pid;
use crate::core::jobtable::JobEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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
    pub db: DataBase,
    pub aliases: HashMap<String, String>,
    pub alias_memo: Vec<(String, String)>,
    pub rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
    pub sigint: Arc<AtomicBool>,
    pub trapped: Vec<(Arc<AtomicBool>, String)>,
    pub traplist: Vec<(i32, String)>,
    pub is_subshell: bool,
    pub source_function_level: i32,
    pub source_files: Vec<String>,
    pub eval_level: i32,
    pub loop_level: i32,
    pub break_counter: i32,
    pub continue_counter: i32,
    pub return_flag: bool,
    pub compat_bash: bool,
    pub tty_fd: Option<OwnedFd>,
    pub job_table: Vec<JobEntry>,
    pub job_table_priority: Vec<usize>,
    current_dir: Option<path::PathBuf>, // the_current_working_directory
    pub completion: Completion,
    pub measured_time: MeasuredTime,
    pub options: Options,
    pub shopts: Options,
    pub suspend_e_option: bool,
    pub script_name: String,
    pub exit_script: String,
    pub exit_script_run: bool,
}

impl ShellCore {
    pub fn configure(&mut self) {
        self.init_current_directory();
        self.set_initial_parameters();
        self.set_builtins();
        signal::ignore(Signal::SIGPIPE);
        signal::ignore(Signal::SIGTSTP);

        let _ = self.db.set_param("PS4", "+ ", None);

        if unistd::isatty(0) == Ok(true) && self.script_name == "-" {
            self.db.flags += "himH";
            let _ = self.db.set_param("PS1", "🍣 ", None);
            let _ = self.db.set_param("PS2", "> ", None);
            let fd = fcntl::fcntl(0, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            self.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }else{
            self.db.flags += "h";
        }

        let home = self.db.get_param("HOME").unwrap_or(String::new()).to_string();
        let _ = self.db.set_param("HISTFILE", &(home + "/.sush_history"), None);
        let _ = self.db.set_param("HISTFILESIZE", "2000", None);

        match env::var("SUSH_COMPAT_TEST_MODE").as_deref() {
            Ok("1") => {
                if self.db.flags.contains('i') {
                    eprintln!("THIS IS BASH COMPATIBILITY TEST MODE");
                }
                self.compat_bash = true;
            },
            _ => {},
        };
    }

    pub fn new() -> Self {
        ShellCore{
            db: DataBase::new(),
            sigint: Arc::new(AtomicBool::new(false)),
            options: Options::new_as_basic_opts(),
            shopts: Options::new_as_shopts(),
            script_name: "-".to_string(),
            ..Default::default()
        }
    }

    pub fn configure_c_mode(&mut self) {
        if unistd::isatty(0) == Ok(true) {
            let fd = fcntl::fcntl(0, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            self.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }

        self.init_current_directory();
        self.set_initial_parameters();
        self.set_builtins();
        signal::ignore(Signal::SIGPIPE);
        signal::ignore(Signal::SIGTSTP);

        match env::var("SUSH_COMPAT_TEST_MODE").as_deref() {
            Ok("1") => self.compat_bash = true,
            _ => {},
        };
    }

    fn set_initial_parameters(&mut self) {
        let version = env!("CARGO_PKG_VERSION");
        let profile = env!("CARGO_BUILD_PROFILE");
        let t_arch = env!("CARGO_CFG_TARGET_ARCH");
        let t_vendor = env!("CARGO_CFG_TARGET_VENDOR");
        let t_os = env!("CARGO_CFG_TARGET_OS");
        let machtype = format!("{}-{}-{}", t_arch, t_vendor, t_os);
        let symbol = "rusty_bash";
        let vparts = version.split('.').collect();
        let versinfo = [vparts, vec![symbol, profile, &machtype]].concat()
                       .iter().map(|e| e.to_string()).collect();

        let _ = self.db.set_param("BASH_VERSION", &format!("{}({})-{}", version, symbol, profile), None);
        let _ = self.db.set_param("MACHTYPE", &machtype, None);
        let _ = self.db.set_param("HOSTTYPE", &t_arch, None);
        let _ = self.db.set_param("OSTYPE", &t_os, None);
        let _ = self.db.set_array("BASH_VERSINFO", versinfo, None);
    }

    pub fn flip_exit_status(&mut self) {
        self.db.exit_status = if self.db.exit_status == 0 { 1 } else { 0 };
    }

    pub fn run_builtin(&mut self, args: &mut Vec<String>, substitutions: &Vec<Substitution>)
    -> Result<bool, ExecError> {
        if args.is_empty() {
            eprintln!("ShellCore::run_builtin");
            return Ok(false);
        }

        if ! self.builtins.contains_key(&args[0]) {
            return Ok(false);
        }

        let mut special_args = vec![];
        for sub in substitutions {
            match args[0].as_ref() {
                "eval" | "declare" => special_args.push(sub.get_string_for_eval(self)?),
                _ => special_args.push(sub.text.clone()),
            }
        }

        let func = self.builtins[&args[0]];
        args.append(&mut special_args);
        self.db.exit_status = func(self, args);
        Ok(true)
    }

    pub fn run_function(&mut self, args: &mut Vec<String>) -> bool {
        match self.db.functions.get_mut(&args[0]) {
            Some(f) => {f.clone().run_as_command(args, self); true},
            None => false,
        }
    }

    fn set_subshell_parameters(&mut self) -> Result<(), String> {
        let pid = nix::unistd::getpid();
        self.db.set_param("BASHPID", &pid.to_string(), Some(0))?;
        match self.db.get_param("BASH_SUBSHELL").unwrap().parse::<usize>() {
            Ok(num) => self.db.set_param("BASH_SUBSHELL", &(num+1).to_string(), Some(0))?,
            Err(_) =>  self.db.set_param("BASH_SUBSHELL", "0", Some(0))?,
        }
        Ok(())
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        signal::restore(Signal::SIGINT);
        signal::restore(Signal::SIGTSTP);
        signal::restore(Signal::SIGPIPE);

        self.is_subshell = true;
        proc_ctrl::set_pgid(self, pid, pgid);
        let _ = self.set_subshell_parameters();
        //self.job_table.clear();

        self.exit_script.clear();
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
        env::set_current_dir(path)?;
        self.current_dir = Some(path.clone());
        Ok(())
    }

    pub fn get_ps4(&mut self) -> String {
        let ps4 = self.db.get_param("PS4").unwrap_or_default().trim_end().to_string();
        let mut multi_ps4 = ps4.to_string();
        for _ in 0..(self.source_files.len() as i32 + self.eval_level) {
            multi_ps4 += &ps4;
        }

        multi_ps4
    }

    pub fn replace_alias(&mut self, word: &mut String) -> bool {
        let before = word.clone();
        match self.replace_alias_core(word) {
            true => {
                self.alias_memo.push( (before, word.clone()) );
                true
            },
            false => false,
        }
    }

    fn replace_alias_core(&self, word: &mut String) -> bool {
        if ! self.shopts.query("expand_aliases") {
            if ! self.db.flags.contains('i') {
                return false;
            }
        }

        let mut ans = false;
        let mut prev_head = "".to_string();
        let history = vec![word.clone()];

        loop {
            let head = match word.replace("\n", " ").split(' ').nth(0) {
                Some(h) => h.to_string(),
                _ => return ans,
            };

            if prev_head == head {
                return ans;
            }
    
            if let Some(value) = self.aliases.get(&head) {
                *word = word.replacen(&head, value, 1);
                if history.contains(word) {
                    return false;
                }
                ans = true;
            }
            prev_head = head;
        }
    }
}
