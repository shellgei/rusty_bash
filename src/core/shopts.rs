//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug)]
pub struct Shopts {
    opts: HashMap<String, bool>,
}

impl Shopts {
    pub fn new() -> Shopts {
        let mut shopts = Shopts {
            opts: HashMap::new(),
        };

        /*
        let opt_strs = vec!["autocd", "cdable_vars", "cdspell", "checkhash",
                   "checkjobs", "checkwinsize", "cmdhist", "compat31",
                   "compat32", "compat40", "compat41", "dirspell",
                   "dotglob", "execfail", "expand_aliases", "extdebug",
                   "extglob", "extquote", "failglob", "force_fignore",
                   "globstar", "gnu_errfmt", "histappend", "histreedit",
                   "histverify", "hostcomplete", "huponexit", "interactive_comments",
                   "lastpipe", "lithist", "login_shell", "mailwarn",
                   "no_empty_cmd_completion", "nocaseglob", "nocasematch", "nullglob",
                   "progcomp", "promptvars", "restricted_shell", "shift_verbose",
                   "sourcepath", "xpg_echo"];

        for opt in opt_strs {
            shopts.opts.insert(opt.to_string(), false);
        }*/

        shopts.opts.insert("extglob".to_string(), true);

        shopts
    }

    pub fn print_all(&self) {
        let mut list = vec![];

        for opt in &self.opts {
            let onoff = match opt.1 {
                true  => "on",
                false => "off",
            };
            if opt.0.len() < 16 {
                list.push(format!("{:16}{}", opt.0, onoff));
            }else{
                list.push(format!("{}\t{}", opt.0, onoff));
            }
        }

        list.sort();
        list.iter().for_each(|e| println!("{}", e));
    }

    pub fn print_if(&self, onoff: bool) {
        let mut list = vec![];

        let onoff_str = match onoff {
            true  => "on",
            false => "off",
        };

        for opt in &self.opts {
            if *opt.1 != onoff {
                continue;
            }

            if opt.0.len() < 16 {
                list.push(format!("{:16}{}", opt.0, onoff_str));
            }else{
                list.push(format!("{}\t{}", opt.0, onoff_str));
            }
        }

        list.sort();
        list.iter().for_each(|e| println!("{}", e));
    }

    pub fn query(&self, opt: &str) -> bool {
        self.opts[opt]
    }

    pub fn set(&mut self, opt: &str, onoff: bool) {
        self.opts.insert(opt.to_string(), onoff);
    }
}
