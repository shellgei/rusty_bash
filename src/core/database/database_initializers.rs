//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::{Data, IntData, Uninit, AssocData, IntAssocData, ArrayData, IntArrayData, OnDemandArray};
use crate::error::exec::ExecError;
use crate::utils;
use crate::utils::clock;
use super::data::single_ondemand::OnDemandSingle;
use super::data::random::RandomVar;
use super::data::seconds::Seconds;
use super::data::srandom::SRandomVar;
use std::{env, process};
use nix::unistd;

impl DataBase {
    pub(super) fn initialize(&mut self) -> Result<(), String> {
        self.exit_status = 0;
    
        self.set_param("$", &process::id().to_string(), None)?;
        self.set_param("BASHPID", &process::id().to_string(), None)?;
        self.set_flag("BASHPID", 'i', 0);
        self.set_param("BASH_SUBSHELL", "0", None)?;
        self.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
        self.set_param("OPTIND", "1", None)?;
        self.set_flag("BASHPID", 'i', 0);
        self.set_param("IFS", " \t\n", None)?;
    
        self.init_as_num("UID", &unistd::getuid().to_string(), None)?;
        self.set_flag("UID", 'i', 0);
        self.set_flag("UID", 'r', 0);
    
        self.params[0].insert("RANDOM".to_string(), Box::new(RandomVar::new()));
        self.params[0].insert("SRANDOM".to_string(), Box::new(SRandomVar::new()));
        self.params[0].insert("SECONDS".to_string(), Box::new(Seconds::new()));
        self.params[0].insert("EPOCHSECONDS".to_string(), Box::new(OnDemandSingle::new(clock::get_epochseconds)));
        self.params[0].insert("EPOCHREALTIME".to_string(), Box::new(OnDemandSingle::new(clock::get_epochrealtime)));
        self.params[0].insert("GROUPS".to_string(), Box::new(OnDemandArray::new(utils::groups)));

        self.init_array("BASH_SOURCE", Some(vec![]), None, false)?;
        self.init_array("BASH_ARGC", Some(vec![]), None, false)?;
        self.init_array("BASH_ARGV", Some(vec![]), None, false)?;
        self.init_array("BASH_LINENO", Some(vec![]), None, false)?;
        self.init_array("DIRSTACK", Some(vec![]), None, false)?;

        self.init_assoc("BASH_ALIASES", None, true, false)?;
        self.init_assoc("BASH_CMDS", None, true, false)?;

        Ok(())
    }

    pub fn init_as_num(&mut self, name: &str, value: &str, scope: Option<usize>,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &Some(vec![]))?;

        let mut data = IntData::new();

        if !value.is_empty() {
            match value.parse::<isize>() {
                Ok(n) => data.body = n,
                Err(e) => return Err(ExecError::Other(e.to_string())),
            }
        }

        let scope = self.get_target_scope(name, scope);
        self.set_entry(scope, name, Box::new(data))
    }

    pub fn init_array(
        &mut self,
        name: &str,
        v: Option<Vec<String>>,
        scope: Option<usize>,
        i_flag: bool,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &v)?;

        let scope = self.get_target_scope(name, scope);
        if i_flag {
            let mut obj = IntArrayData::new();
            if let Some(v) = v {
                for (i, e) in v.into_iter().enumerate() {
                    obj.set_as_array(name, &i.to_string(), &e)?;
                }
            }
            return self.set_entry(scope, name, Box::new(obj));
        }

        if v.is_none() {
            return self.set_entry(scope, name, Box::new(Uninit::new("a")));
        }

        let mut obj = ArrayData::new();
        for (i, e) in v.unwrap().into_iter().enumerate() {
            obj.set_as_array(name, &i.to_string(), &e)?;
        }
        self.set_entry(scope, name, Box::new(obj))
    }

    pub fn init_assoc(
        &mut self,
        name: &str,
        scope: Option<usize>,
        set_array: bool,
        i_flag: bool,
    ) -> Result<(), ExecError> {
        self.check_on_write(name, &None)?;

        let obj = if i_flag {
            Box::new(IntAssocData::new()) as Box::<dyn Data>
        } else if set_array {
            Box::new(AssocData::new()) as Box::<dyn Data>
        } else {
            Box::new(Uninit::new("A")) as Box::<dyn Data>
        };

        let scope = self.get_target_scope(name, scope);
        self.set_entry(scope, name, obj)
    }
}
