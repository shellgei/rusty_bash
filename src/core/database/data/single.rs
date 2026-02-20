//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{case_change, Data};
use crate::error::exec::ExecError;
use crate::utils;

#[derive(Debug, Clone)]
pub struct SingleData {
    pub body: String,
    pub flags: String,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            body: s.to_string(),
            flags: String::new(),
        }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_fmt_string(&mut self) -> String {
        self._get_fmt_string()
    }

    fn _get_fmt_string(&self) -> String {
        let mut s = self.body.replace("'", "\\'");
        if s.contains('~') || s.starts_with('#') {
            s = "'".to_owned() + &s + "'";
        }
        let ansi = utils::to_ansi_c(&s);
        if ansi == s {
            ansi.replace("$", "\\$")
        } else {
            ansi
        }
    }

    fn clear(&mut self) {
        self.body.clear();
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        self.nameref_check(name, value)?;

        /*
        if self.has_flag('n') {
            if value.contains('[') {
                let splits: Vec<&str> = value.split('[').collect();
                if ! utils::is_var(&splits[0]) || ! splits[1].ends_with(']') {
                        return Err(ExecError::InvalidNameRef(value.to_string()));
                }

                if name == splits[0] {
                        return Err(ExecError::SelfRef(name.to_string()));
                }
            }else if value == "" {
            }else if ! utils::is_var(value) {
                return Err(ExecError::InvalidNameRef(value.to_string()));
            }else if name == value {
                return Err(ExecError::SelfRef(name.to_string()));
            }
        }*/

        self.body = value.to_string();
        case_change(&self.flags, &mut self.body);
        Ok(())
    }

    fn append_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        self.body += value;
        case_change(&self.flags, &mut self.body);
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(self.body.to_string())
    }

    fn len(&mut self) -> usize {
        self.body.chars().count()
    }

    fn is_single(&self) -> bool {
        true
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        Ok(key == "0")
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn unset_flag(&mut self, flag: char) {
        self.flags.retain(|e| e != flag);
    }

    fn get_flags(&mut self) -> &str {
        &self.flags
    }
}

impl SingleData {
    pub fn new(flags: &str) -> Self {
        Self {
            body: "".to_string(),
            flags: flags.to_string(),
        }
    }
}
