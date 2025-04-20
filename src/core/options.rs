//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Options {
    opts: HashMap<String, bool>,
}

impl Options {
    pub fn new_as_basic_opts() -> Options {
        let mut options = Options::default();
        options.opts.insert("pipefail".to_string(), false);
        options.opts.insert("noglob".to_string(), false);
        options.opts.insert("posix".to_string(), false); //TODO: still dummy
        options
    }

    pub fn new_as_shopts() -> Options {
        let mut options = Options::default();
        let opt_strs = vec!["autocd", "cdable_vars", "cdspell", "checkhash",
                   "checkjobs", "checkwinsize", "cmdhist", "compat31",
                   "compat32", "compat40", "compat41", "dirspell",
                   "dotglob", "execfail", "expand_aliases", "extdebug",
                   "extglob", "extquote", "failglob", "force_fignore",
                   "globstar", "globskipdots", "gnu_errfmt", "histappend", "histreedit",
                   "histverify", "hostcomplete", "huponexit", "interactive_comments",
                   "lastpipe", "lithist", "login_shell", "mailwarn",
                   "no_empty_cmd_completion", "nocaseglob", "nocasematch", "nullglob",
                   "promptvars", "restricted_shell", "shift_verbose",
                   "sourcepath", "xpg_echo"];

        for opt in opt_strs {
            options.opts.insert(opt.to_string(), false);
        }

        let true_list = ["extglob", "progcomp", "globskipdots"];
        for opt in true_list {
            options.opts.insert(opt.to_string(), true);
        }

        options
    }

    pub fn format(opt: &str, onoff: bool) -> String {
        let onoff_str = match onoff {
            true  => "on",
            false => "off",
        };

        match opt.len() < 16 {
            true  => format!("{:16}{}", opt, onoff_str),
            false => format!("{}\t{}", opt, onoff_str), 
        }
    }

    pub fn format2(opt: &str, onoff: bool) -> String {
        let onoff_str = match onoff {
            true  => "-",
            false => "+",
        };

        format!("set {}o {}", onoff_str, opt)
    }

    pub fn print_opt(&self, opt: &str, set_format: bool) -> bool {
        match self.opts.get_key_value(opt) {
            None     => {
                eprintln!("sush: shopt: {}: invalid shell option name", opt);
                false
            },
            Some(kv) => {
                match set_format {
                    false => println!("{}", Self::format(kv.0, *kv.1)),
                    true  => println!("{}", Self::format2(kv.0, *kv.1)),
                }
                true
            },
        }
    }

    pub fn print_all(&self, positive: bool) {
        let f = match positive {
            true => Self::format,
            false => Self::format2,
        };

        let mut list = self.opts.iter()
                       .map(|opt| f(opt.0, *opt.1))
                       .collect::<Vec<String>>();

        list.sort();
        list.iter().for_each(|e| println!("{}", e));
    }

    pub fn print_if(&self, onoff: bool) {
        let mut list = self.opts.iter()
                       .filter(|opt| *opt.1 == onoff)
                       .map(|opt| Self::format(opt.0, *opt.1))
                       .collect::<Vec<String>>();

        list.sort();
        list.iter().for_each(|e| println!("{}", e));
    }

    pub fn exist(&self, opt: &str) -> bool {
        self.opts.contains_key(opt)
    }

    pub fn query(&self, opt: &str) -> bool {
        self.exist(opt) && self.opts[opt]
    }

    pub fn set(&mut self, opt: &str, onoff: bool) -> bool {
        if ! self.opts.contains_key(opt) {
            eprintln!("sush: shopt: {}: invalid shell option name", opt);
            return false;
        }

        self.opts.insert(opt.to_string(), onoff);
        true
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.opts.clone().into_keys().collect()
    }
}
