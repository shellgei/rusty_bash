//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore, utils};
use super::{BracedParam, ExecError, Subword};

impl BracedParam {
    pub(super) fn indirect_preparation(&mut self, core: &mut ShellCore) -> Result<bool, ExecError> {
        if ! core.db.exist(&self.param.name)
        && ! core.db.exist_nameref(&self.param.name) {
            return Err(ExecError::InvalidIndirectExpansion(self.param.name.to_string()));
        }

        if core.db.has_flag(&self.param.name, 'n') {
            if self.text.contains("[") {
                self.text = String::new();
            } else if let Some(nameref) = core.db.get_nameref(&self.param.name)? {
                self.text = nameref;
            }else{
                self.text = String::new();
            }
            return Ok(false);
        }

        if self.param.is_var_array() { // ${!name[@]}, ${!name[*]}
            self.index_replace(core)?;
            return Ok(false);
        }

        self.indirect_replace(core)?;
        self.check()?;
        Ok(true)
    }

    fn indirect_replace(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut sw = self.clone();
        sw.indirect = false;
        sw.unknown = String::new();
        sw.treat_as_array = false;
        sw.num = false;

        sw.substitute(core)?;

        if sw.text.contains('[') {
            let mut feeder = Feeder::new(&("${".to_owned() + &sw.text + "}"));
            if let Ok(Some(mut bp)) = BracedParam::parse(&mut feeder, core) {
                bp.substitute(core)?;
                self.param.name = bp.param.name;
                self.param.index = bp.param.index;
            } else {
                return Err(ExecError::InvalidName(sw.text.clone()));
            }
        } else {
            self.param.name = sw.text.clone();
            self.param.index = None;
        }

        if !utils::is_param(&self.param.name) {
            return Err(ExecError::InvalidName(self.param.name.clone()));
        }
        Ok(())
    }
}
