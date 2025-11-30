//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::IntData;
use crate::error::exec::ExecError;
use super::data::epochrealtime::EpochRealTime;
use super::data::epochseconds::EpochSeconds;
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
        self.set_param("BASH_SUBSHELL", "0", None)?;
        self.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
        self.set_param("OPTIND", "1", None)?;
        self.set_param("IFS", " \t\n", None)?;
    
        self.init_as_num("UID", &unistd::getuid().to_string(), None)?;
        self.set_flag("UID", 'r', 0);
    
        self.params[0].insert("RANDOM".to_string(), Box::new(RandomVar::new()));
        self.params[0].insert("SRANDOM".to_string(), Box::new(SRandomVar::new()));
        self.params[0].insert("SECONDS".to_string(), Box::new(Seconds::new()));
        self.params[0].insert("EPOCHSECONDS".to_string(), Box::new(EpochSeconds::new()));
        self.params[0].insert("EPOCHREALTIME".to_string(), Box::new(EpochRealTime::new()));
    
        self.set_array("FUNCNAME", None, None, false)?;
        self.set_array("BASH_SOURCE", Some(vec![]), None, false)?;
        self.set_array("BASH_ARGC", Some(vec![]), None, false)?;
        self.set_array("BASH_ARGV", Some(vec![]), None, false)?;
        self.set_array("BASH_LINENO", Some(vec![]), None, false)?;
        self.set_array("DIRSTACK", Some(vec![]), None, false)?;
        self.set_assoc("BASH_ALIASES", None, true, false)?;
        self.set_assoc("BASH_CMDS", None, true, false)?;
        Ok(())
    }

    pub fn init_as_num(&mut self, name: &str, value: &str, layer: Option<usize>,
    ) -> Result<(), ExecError> {
        self.write_check(name, &Some(vec![]))?;

        let mut data = IntData::new();

        if !value.is_empty() {
            match value.parse::<isize>() {
                Ok(n) => data.body = n,
                Err(e) => return Err(ExecError::Other(e.to_string())),
            }
        }

        let layer = self.get_target_layer(name, layer);
        self.set_entry(layer, name, Box::new(data))
    }
}
