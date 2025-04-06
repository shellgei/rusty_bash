//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use std::collections::HashMap;
use super::Data;

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    body: HashMap<usize, String>,
}

impl From<Vec<String>> for ArrayData {
    fn from(v: Vec<String>) -> Self {
        let mut ans = Self { body: HashMap::new() };
        v.into_iter()
         .enumerate()
         .for_each(|(i, e)| {ans.body.insert(i, e);});
        ans
    }
}

impl Data for ArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }

    fn print_body(&self) -> String {
        let mut formatted = String::new();
        formatted += "(";
        for i in self.keys() {
            formatted += &format!("[{}]=\"{}\" ", i, &self.body[&i]).clone();
        };
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body.insert(0, value.to_string());
        Ok(())
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        if let Ok(n) = key.parse::<usize>() {
            self.body.insert(n, value.to_string());
            return Ok(());
        }
        Err(ExecError::Other("invalid index".to_string()))
    }

    fn get_as_array(&mut self, key: &str) -> Result<String, ExecError> {
        dbg!("GET_AS_ARRAY");
        if key == "@" || key == "*" {
            return Ok(self.values().join(" "));
        }

        let n = key.parse::<usize>().map_err(|_| ExecError::ArrayIndexInvalid(key.to_string()))?;
        Ok( self.body.get(&n).unwrap_or(&"".to_string()).clone() )
    }

    fn get_all_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }
        
        let keys = self.keys();
        let max = *keys.iter().max().unwrap() as usize;
        let mut ans = vec![];
        for i in 0..(max+1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.clone()),
                None => ans.push("".to_string()),
            }
        }
        Ok(ans)
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().iter().map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        self.body.get(&0).map(|v| Ok(v.clone())).ok_or(ExecError::Other("No entry".to_string()))?
    }

    fn is_array(&self) -> bool {true}
    fn len(&mut self) -> usize { self.body.len() }
}

impl ArrayData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, v: Vec<String>) -> Result<(), ExecError> {
        db_layer.insert(name.to_string(), Box::new(ArrayData::from(v)));
        Ok(())
    }

    pub fn set_elem(db_layer: &mut HashMap<String, Box<dyn Data>>,
                        name: &str, pos: usize, val: &String) -> Result<(), ExecError> {
        match db_layer.get_mut(name) {
            Some(d) => d.set_as_array(&pos.to_string(), val),
            None    => {
                ArrayData::set_new_entry(db_layer, name, vec![])?;
                Self::set_elem(db_layer, name, pos, val)
            },
        }
    }

    pub fn values(&self) -> Vec<String> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys.iter().map(|i| self.body[i].clone()).collect()
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = self.body.iter().map(|e| e.0.clone()).collect();
        keys.sort();
        keys
    }
}
