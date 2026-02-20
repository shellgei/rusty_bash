//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::single::SingleData;
use super::Data;
use crate::error::exec::ExecError;
use crate::utils;

#[derive(Debug, Clone)]
pub struct IntData {
    pub body: isize,
    pub flags: String,
}

impl Data for IntData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn _get_fmt_string(&self) -> String {
        utils::to_ansi_c(&self.body.to_string())
    }

    fn clear(&mut self) {}

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        match value.parse::<isize>() {
            Ok(n) => self.body = n,
            Err(e) => {
                return Err(ExecError::Other(e.to_string()));
            }
        }
        Ok(())
    }

    fn append_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        match value.parse::<isize>() {
            Ok(n) => self.body += n,
            Err(e) => {
                return Err(ExecError::Other(e.to_string()));
            }
        }
        Ok(())
    }

    fn init_as_num(&mut self) -> Result<(), ExecError> {
        self.body = 0;
        Ok(())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok(self.body.to_string())
    }
    fn get_as_single_num(&mut self) -> Result<isize, ExecError> {
        Ok(self.body)
    }

    fn get_str_type(&self) -> Box<dyn Data> {
        let mut d = SingleData::from(self.body.to_string().as_ref());
        d.flags = self.flags.clone();
        let _ = d.unset_flag('i');
        Box::new(d)
    }

    fn len(&mut self) -> usize {
        self.body.to_string().len()
    }
    fn is_single(&self) -> bool {
        true
    }
    fn is_single_num(&self) -> bool {
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

    fn has_flag(&mut self, flag: char) -> bool {
        if flag == 'i' {
            return true;
        }
        self.flags.contains(flag)
    }

    fn get_flags(&mut self) -> &str {
        &self.flags
    }
}

impl IntData {
    pub fn new() -> Self {
        Self {
            body: 0,
            flags: "i".to_string(),
        }
    }
}
