//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug)]
pub struct Shopts {
    shopts: HashMap<String, bool>,
}

impl Shopts {
    pub fn new() -> Shopts {
        let mut shopts = Shopts {
            shopts: HashMap::new(),
        };

        let opts = vec!["autocd", "cdable_vars", "cdspell", "checkhash",
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

        for opt in opts {
            shopts.shopts.insert(opt.to_string(), false);
        }

        shopts.shopts.insert("extglob".to_string(), true);

        shopts
    }
}
