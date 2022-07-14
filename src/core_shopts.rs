//SPDX-FileCopyrightText:
//SPDX-License-Identifier:

use std::collections::HashMap;

pub struct Shopts(HashMap<&'static str, bool>);

impl Shopts {
    pub fn get(self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(ans) => return Some(*ans),
            _ => return None,
        }
    }

    pub fn set(&mut self, key: &'static str, value: bool) -> bool {
        self.0.insert(key, value);
        true
    }

    pub fn new () -> Shopts {
        let mut ans = HashMap::new();
        ans.insert("autocd", false); 
        ans.insert("assoc_expand_once", false); 
        ans.insert("cdable_vars", false); 
        ans.insert("cdspell", false); 
        ans.insert("checkhash", false); 
        ans.insert("checkjobs", false); 
        ans.insert("checkwinsize", false); 
        ans.insert("cmdhist", false); 
        ans.insert("compat31", false); 
        ans.insert("compat32", false); 
        ans.insert("compat40", false); 
        ans.insert("compat41", false); 
        ans.insert("compat42", false); 
        ans.insert("compat43", false); 
        ans.insert("compat44", false); 
        ans.insert("complete_fullquote", false); 
        ans.insert("direxpand", false); 
        ans.insert("dirspell", false); 
        ans.insert("dotglob", false); 
        ans.insert("execfail", false); 
        ans.insert("expand_aliases", false); 
        ans.insert("extdebug", false); 
        ans.insert("extglob", false); 
        ans.insert("extquote", false); 
        ans.insert("failglob", false); 
        ans.insert("force_fignore", false); 
        ans.insert("globasciiranges", false); 
        ans.insert("globstar", false); 
        ans.insert("gnu_errfmt", false); 
        ans.insert("histappend", false); 
        ans.insert("histreedit", false); 
        ans.insert("histverify", false); 
        ans.insert("hostcomplete", false); 
        ans.insert("huponexit", false); 
        ans.insert("inherit_errexit", false); 
        ans.insert("interactive_comments", false); 
        ans.insert("lastpipe", false); 
        ans.insert("lithist", false); 
        ans.insert("localvar_inherit", false); 
        ans.insert("localvar_unset", false); 
        ans.insert("login_shell", false); 
        ans.insert("mailwarn", false); 
        ans.insert("no_empty_cmd_completion", false); 
        ans.insert("nocaseglob", false); 
        ans.insert("nocasematch", false); 
        ans.insert("nullglob", false); 
        ans.insert("progcomp", false); 
        ans.insert("progcomp_alias", false); 
        ans.insert("promptvars", false); 
        ans.insert("restricted_shell", false); 
        ans.insert("shift_verbose", false); 
        ans.insert("sourcepath", false); 
        ans.insert("xpg_echo", false); 

        Shopts(ans)
    }
}
