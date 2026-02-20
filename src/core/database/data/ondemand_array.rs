//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use super::{Data, ExecError};

#[derive(Debug, Clone)]
pub struct OnDemandArray {
    pub values: fn() -> Vec<String>,
    pub flags: String,
}

impl Data for OnDemandArray {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn _get_fmt_string(&self) -> String {
        return "*********".to_string()
    }

    fn get_fmt_string(&mut self) -> String {
        let mut formatted = "(".to_string();
        for (i, v) in (self.values)().into_iter().enumerate() {
            let ansi = utils::to_ansi_c(&v);
            if ansi == v {
                formatted += &format!("[{}]=\"{}\" ", i, &ansi.replace("$", "\\$"));
            } else {
                formatted += &format!("[{}]={} ", i, &ansi);
            }
        }
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok((self.values)().join(" "));
        }
        if key == "*" {
            return Ok((self.values)().join(ifs));
        }

        let index = self.index_of(key)?;
        let vs = (self.values)();
        if index < vs.len() {
            return Ok(vs[index].clone());
        }

        Ok("".to_string())
    }

    fn get_vec_from(&mut self, pos: usize, _: bool) -> Result<Vec<String>, ExecError> {
        let vs = (self.values)();
        if pos < vs.len() {
            return Ok(vs[pos..].to_vec());
        }

        Ok(vec![])
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        let num = (self.values)().len();
        Ok((0..num).map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let vs = (self.values)();
        if vs.is_empty() {
            Ok("".to_string())
        }else{
            Ok(vs[0].clone())
        }
    }

    fn is_array(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        (self.values)().len()
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }

        let n = self.index_of(key)?;
        Ok(n < (self.values)().len())
    }

    fn index_based_len(&mut self) -> usize {
        self.len()
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = self.index_of(key)?;
        let vs = (self.values)();

        if n < vs.len() {
            return Ok(vs[n].to_string().len());
        }

        Ok(0)
    }

    fn remove_elem(&mut self, _: &str) -> Result<(), ExecError> {
        Ok(())
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
        self.flags.contains(flag)
    }

    fn get_flags(&mut self) -> String {
        self.flags.clone()
    }
}

impl OnDemandArray {
    pub fn new(values: fn() -> Vec<String>) -> Self {
        Self {
            values: values,
            flags: "a".to_string(),
        }
    }

    fn index_of(&mut self, key: &str) -> Result<usize, ExecError> {
        let index = match key.parse::<isize>() {
            Ok(i) => i,
            _ => return Err(ExecError::ArrayIndexInvalid(key.to_string())),
        };

        Ok(index as usize)
    }
}
