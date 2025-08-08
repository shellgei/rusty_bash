//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use crate::utils;

#[derive(Debug, Clone)]
pub struct SingleData {
    body: String,
}

impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            body: s.to_string(),
        }
    }
}

impl Data for SingleData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn print_body(&self) -> String {
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

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body = value.to_string();
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body += value;
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
}
