//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::{BracedParam, ExecError};
use crate::ShellCore;

impl BracedParam {
    pub fn subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let index = self
            .param
            .index
            .clone()
            .unwrap()
            .eval(core, &self.param.name)?;

        if self.num {
            self.text = core.db.get_elem_len(&self.param.name, &index)?.to_string();
            return Ok(());
        }

        if core.db.is_single(&self.param.name) {
            let param = core.db.get_param(&self.param.name)?;
            let tmp = match index.as_str() {
                //case: a=aaa; echo ${a[@]}; (output: aaa)
                "@" | "*" | "0" => param, //.unwrap_or("".to_string()),
                _ => "".to_string(),
            };
            self.text = self.extension(tmp, core)?;
            return Ok(());
        }

        let ifs = core.db.get_ifs_head();

        if index.as_str() == "@" {
            self.atmark_operation(core, " ")
        } else if index.as_str() == "*" {
            self.atmark_operation(core, &ifs)
        } else {
            let tmp = core.db.get_elem(&self.param.name, &index)?;
            self.text = self.extension(tmp, core)?;
            Ok(())
        }
    }

    fn atmark_operation(&mut self, core: &mut ShellCore, ifs: &str) -> Result<(), ExecError> {
        let mut arr = core.db.get_vec(&self.param.name, true)?;
        self.array = Some(arr.clone());
        if self.num {
            self.text = arr.len().to_string();
            return Ok(());
        }

        self.text = match self.num {
            true => core.db.get_var_len(&self.param.name).to_string(),
            false => core.db.get_vec(&self.param.name, true)?.join(ifs),
        };

        if arr.len() <= 1 || self.has_value_check() {
            self.text = self.extension(self.text.clone(), core)?;
        } else {
            for item in arr.iter_mut() {
                *item = self.extension(item.clone(), core)?;
            }
            self.text = arr.join(ifs);
            self.array = Some(arr);
        }
        Ok(())
    }

    fn has_value_check(&mut self) -> bool {
        match self.extension.as_mut() {
            Some(op) => op.is_value_check(),
            _ => false,
        }
    }

    fn extension(&mut self, text: String, core: &mut ShellCore) -> Result<String, ExecError> {
        match self.extension.as_mut() {
            Some(op) => op.exec(&self.param, &text, core),
            None => Ok(text.clone()),
        }
    }
}
