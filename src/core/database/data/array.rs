//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{case_change, Data};
use crate::error::exec::ExecError;
use crate::utils;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ArrayData {
    pub body: HashMap<usize, String>,
    pub flags: String,
}

impl From<Option<Vec<String>>> for ArrayData {
    fn from(v: Option<Vec<String>>) -> Self {
        let mut ans = Self::new();
        v.unwrap().into_iter().enumerate().for_each(|(i, e)| {
            ans.body.insert(i, e);
        });
        ans
    }
}

impl Data for ArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_print_string(&mut self) -> String {
        self.get_print_string_fix()
    }

    fn get_print_string_fix(&self) -> String {
        let mut formatted = "(".to_string();
        for i in self.keys() {
            let ansi = utils::to_ansi_c(&self.body[&i]);
            if ansi == self.body[&i] {
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

    fn clear(&mut self) {
        self.body.clear();
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        let mut value = value.to_string();
        case_change(&self.flags, &mut value);
        self.body.insert(0, value);
        Ok(())
    }

    fn append_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        let mut value = if let Some(v) = self.body.get(&0) {
            v.to_owned() + value
        } else {
            value.to_string()
        };

        case_change(&self.flags, &mut value);
        self.body.insert(0, value);
        Ok(())
    }

    fn set_as_array(&mut self, name: &str, key: &str,
                    value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        let n = self.index_of(key)?;

        let mut value = value.to_string();
        case_change(&self.flags, &mut value);

        self.body.insert(n, value);
        Ok(())
    }

    fn append_to_array_elem(&mut self, name: &str, key: &str,
                            value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;
        let n = self.index_of(key)?;
        let mut value = if let Some(v) = self.body.get(&n) {
            v.to_owned() + value
        } else {
            value.to_string()
        };

        case_change(&self.flags, &mut value);
        self.body.insert(n, value);
        Ok(())
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok(self.values().join(" "));
        }
        if key == "*" {
            return Ok(self.values().join(ifs));
        }

        let n = self.index_of(key)?;
        Ok(self.body.get(&n).unwrap_or(&"".to_string()).clone())
    }

    fn get_vec_from(&mut self, pos: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap();
        let mut ans = vec![];
        for i in pos..(max + 1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.clone()),
                None => {
                    if !skip_non {
                        ans.push("".to_string());
                    }
                }
            }
        }
        Ok(ans)
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().iter().map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        self.body
            .get(&0)
            .map(|v| Ok(v.clone()))
            .ok_or(ExecError::Other("No entry".to_string()))?
    }

    fn is_array(&self) -> bool {
        true
    }
    fn len(&mut self) -> usize {
        self.body.len()
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        let n = self.index_of(key)?;
        Ok(self.body.contains_key(&n))
    }

    fn index_based_len(&mut self) -> usize {
        match self.body.iter().map(|e| e.0).max() {
            Some(n) => *n + 1,
            None => 0,
        }
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = self.index_of(key)?;
        let s = self.body.get(&n).unwrap_or(&"".to_string()).clone();

        Ok(s.chars().count())
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            self.body.clear();
            return Ok(());
        }

        let index = self.index_of(key)?;
        self.body.remove(&index);
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
}

impl ArrayData {
    pub fn new() -> Self {
        Self {
            body: HashMap::new(),
            flags: "a".to_string(),
        }
    }

    pub fn values(&self) -> Vec<String> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| *e.0).collect();
        keys.sort();
        keys.iter().map(|i| self.body[i].clone()).collect()
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| *e.0).collect();
        keys.sort();
        keys
    }

    fn index_of(&mut self, key: &str) -> Result<usize, ExecError> {
        let mut index = match key.parse::<isize>() {
            Ok(i) => i,
            _ => return Err(ExecError::ArrayIndexInvalid(key.to_string())),
        };

        if index >= 0 {
            return Ok(index as usize);
        }

        let keys = self.keys();
        let max = match keys.iter().max() {
            Some(n) => *n as isize,
            None => -1,
        };
        index += max + 1;

        if index < 0 {
            return Err(ExecError::ArrayIndexInvalid(key.to_string()));
        }

        Ok(index as usize)
    }
}
