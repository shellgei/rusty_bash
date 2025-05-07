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
use crate::feeder::terminal::Terminal;

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
    pub editor: Option<Terminal>,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            db: DataBase::new(),
            sigint: Arc::new(AtomicBool::new(false)),
            //read_stdin: true,
            options: Options::new_as_basic_opts(),
            shopts: Options::new_as_shopts(),
            script_name: "-".to_string(),
            ..Default::default()
        };

        core.init_current_directory();
        core.set_initial_parameters();
        core.set_builtins();
        signal::ignore(Signal::SIGPIPE);
        signal::ignore(Signal::SIGTSTP);

        let _ = core.db.set_param("PS4", "+ ", None);

        if unistd::isatty(0) == Ok(true) {
            core.db.flags += "imH";
            //core.read_stdin = false;
            let _ = core.db.set_param("PS1", "🍣 ", None);
            let _ = core.db.set_param("PS2", "> ", None);
            let fd = fcntl::fcntl(0, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            core.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }

        let home = core.db.get_param("HOME").unwrap_or(String::new()).to_string();
        let _ = core.db.set_param("HISTFILE", &(home + "/.sush_history"), None);
        let _ = core.db.set_param("HISTFILESIZE", "2000", None);

        match env::var("SUSH_COMPAT_TEST_MODE").as_deref() {
            Ok("1") => {
                if core.db.flags.contains('i') {
                    eprintln!("THIS IS BASH COMPATIBILITY TEST MODE");
                }
                core.compat_bash = true;
            },
            _ => {},
        };

        if core.db.flags.contains('i') {
            core.editor = Some(Terminal::new(&mut core));
        }

        core
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

    pub fn run_builtin(&mut self, args: &mut Vec<String>, special_args: &mut Vec<String>) -> bool {
        if args.is_empty() {
            eprintln!("ShellCore::run_builtin");
            return false;
        }

        if self.builtins.contains_key(&args[0]) {
            let func = self.builtins[&args[0]];
            args.append(special_args);
            self.db.exit_status = func(self, args);
            return true;
        }

        false
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
        self.job_table.clear();

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

    pub fn write_history_to_file(&mut self) {
        if let (Some(term), Ok(histfile)) = (self.editor.as_mut(), self.db.get_param("HISTFILE")) {
            if !histfile.is_empty() {
                let path = path::PathBuf::from(&histfile);
                term.save_history(&path);
            }
        }
    }
}
