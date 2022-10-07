//SPDX-FileCopyrightText:
//SPDX-License-Identifier:

use std::collections::HashMap;

pub struct Shopts(HashMap<String, bool>);

impl Shopts {
    pub fn _get(self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(ans) => return Some(*ans),
            _ => return None,
        }
    }

    pub fn set(&mut self, key: &String, value: bool) -> bool {
        match self.0.get(key) {
            Some(_) => {
                self.0.insert(key.to_string(), value);
                true
            },
            _ => {
                eprintln!("bash: shopt: {}: invalid shell option name", key);
                false
            },
        }
    }

    pub fn print(&self, on_print: bool, off_print: bool) {
        let mut keys = vec![];
        for k in self.0.keys() {
            keys.push(k);
        }

        keys.sort();
        for k in keys { 
            let output;
            let onoff = if self.0[k] {
                output = on_print;
                "on"
            }else{
                output = off_print;
                "off"
            };

            if output {
                println!("{}\t{}", k, onoff);
            }
        }
    }

    pub fn new () -> Shopts {
        let mut ans = HashMap::new();
        ans.insert("autocd".to_string(), false); 
        ans.insert("assoc_expand_once".to_string(), false); 
        ans.insert("cdable_vars".to_string(), false); 
        ans.insert("cdspell".to_string(), false); 
        ans.insert("checkhash".to_string(), false); 
        ans.insert("checkjobs".to_string(), false); 
        ans.insert("checkwinsize".to_string(), false); 
        ans.insert("cmdhist".to_string(), false); 
        ans.insert("compat31".to_string(), false); 
        ans.insert("compat32".to_string(), false); 
        ans.insert("compat40".to_string(), false); 
        ans.insert("compat41".to_string(), false); 
        ans.insert("compat42".to_string(), false); 
        ans.insert("compat43".to_string(), false); 
        ans.insert("compat44".to_string(), false); 
        ans.insert("complete_fullquote".to_string(), false); 
        ans.insert("direxpand".to_string(), false); 
        ans.insert("dirspell".to_string(), false); 
        ans.insert("dotglob".to_string(), false); 
        ans.insert("execfail".to_string(), false); 
        ans.insert("expand_aliases".to_string(), false); 
        ans.insert("extdebug".to_string(), false); 
        ans.insert("extglob".to_string(), false); 
        ans.insert("extquote".to_string(), false); 
        ans.insert("failglob".to_string(), false); 
        ans.insert("force_fignore".to_string(), false); 
        ans.insert("globasciiranges".to_string(), false); 
        ans.insert("globstar".to_string(), false); 
        ans.insert("gnu_errfmt".to_string(), false); 
        ans.insert("histappend".to_string(), false); 
        ans.insert("histreedit".to_string(), false); 
        ans.insert("histverify".to_string(), false); 
        ans.insert("hostcomplete".to_string(), false); 
        ans.insert("huponexit".to_string(), false); 
        ans.insert("inherit_errexit".to_string(), false); 
        ans.insert("interactive_comments".to_string(), false); 
        ans.insert("lastpipe".to_string(), false); 
        ans.insert("lithist".to_string(), false); 
        ans.insert("localvar_inherit".to_string(), false); 
        ans.insert("localvar_unset".to_string(), false); 
        ans.insert("login_shell".to_string(), false); 
        ans.insert("mailwarn".to_string(), false); 
        ans.insert("no_empty_cmd_completion".to_string(), false); 
        ans.insert("nocaseglob".to_string(), false); 
        ans.insert("nocasematch".to_string(), false); 
        ans.insert("nullglob".to_string(), false); 
        ans.insert("progcomp".to_string(), false); 
        ans.insert("progcomp_alias".to_string(), false); 
        ans.insert("promptvars".to_string(), false); 
        ans.insert("restricted_shell".to_string(), false); 
        ans.insert("shift_verbose".to_string(), false); 
        ans.insert("sourcepath".to_string(), false); 
        ans.insert("xpg_echo".to_string(), false); 

        Shopts(ans)
    }
}
