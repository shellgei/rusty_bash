//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::utils;
use crate::error::exec::ExecError;
use std::collections::HashMap;
use super::Data;
use super::array_uninit::UninitArray;

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    body: HashMap<usize, String>,
}

impl From<Option<Vec<String>>> for ArrayData {
    fn from(v: Option<Vec<String>>) -> Self {
        let mut ans = Self::default();
        v.unwrap().into_iter()
         .enumerate()
         .for_each(|(i, e)| {ans.body.insert(i, e);});
        ans
    }
}

impl Data for ArrayData {
    fn boxed_clone(&self) -> Box<dyn Data> { Box::new(self.clone()) }

    fn print_body(&self) -> String {
        let mut formatted = "(".to_string();
        for i in self.keys() {
            let ansi = utils::to_ansi_c(&self.body[&i]);
            if ansi == self.body[&i] {
                formatted += &format!("[{}]=\"{}\" ", i, &ansi);
            }else{
                formatted += &format!("[{}]={} ", i, &ansi);
            }
        };
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        formatted
    }

    fn clear(&mut self) { self.body.clear(); }
    fn is_initialized(&self) -> bool { true }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        self.body.insert(0, value.to_string());
        Ok(())
    }

    fn append_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        if let Some(v) = self.body.get(&0) {
            self.body.insert(0, v.to_owned() + value);
        }else {
            self.body.insert(0, value.to_string());
        }
        Ok(())
    }

    fn set_as_array(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        if let Ok(n) = key.parse::<usize>() {
            self.body.insert(n, value.to_string());
            return Ok(());
        }
        Err(ExecError::Other("invalid index".to_string()))
    }

    /*
    fn push_elems(&mut self, values: Vec<String>) -> Result<(), ExecError> {
        let mut index = match self.body.is_empty() {
            true  => 0,
            false => *self.keys().iter().max().unwrap(),
        };

        for v in values {
            self.body.insert(index, v);
            index += 1;
        }
        Ok(())
    }*/

    fn append_to_array_elem(&mut self, key: &str, value: &str) -> Result<(), ExecError> {
        if let Ok(n) = key.parse::<usize>() {
            if let Some(v) = self.body.get(&n) {
                self.body.insert(n, v.to_owned() + value);
            }else{
                self.body.insert(n, value.to_string());
            }
            return Ok(());
        }
        Err(ExecError::Other("invalid index".to_string()))
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok(self.values().join(" "));
        }
        if key == "*" {
            return Ok(self.values().join(ifs));
        }

        let n = key.parse::<usize>().map_err(|_| ExecError::ArrayIndexInvalid(key.to_string()))?;
        Ok( self.body.get(&n).unwrap_or(&"".to_string()).clone() )
    }

    /*
    fn get_all_as_array(&mut self, skip_non: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap() as usize;
        let mut ans = vec![];
        for i in 0..(max+1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.clone()),
                None => {
                    if ! skip_non {
                        ans.push("".to_string());
                    }
                },
            }
        }
        Ok(ans)
    }
    */

    fn get_array_from(&mut self, pos: usize, skip_non: bool) -> Result<Vec<String>, ExecError> {
        if self.body.is_empty() {
            return Ok(vec![]);
        }

        let keys = self.keys();
        let max = *keys.iter().max().unwrap() as usize;
        let mut ans = vec![];
        for i in pos..(max+1) {
            match self.body.get(&i) {
                Some(s) => ans.push(s.clone()),
                None => {
                    if ! skip_non {
                        ans.push("".to_string());
                    }
                },
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

    fn index_based_len(&mut self) -> usize {
        match self.body.iter().map(|e| e.0).max() {
            Some(n) => *n+1,
            None    => 0,
        }
    }

    fn elem_len(&mut self, key: &str) -> Result<usize, ExecError> {
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = key.parse::<usize>().map_err(|_| ExecError::ArrayIndexInvalid(key.to_string()))?;
        let s = self.body.get(&n).unwrap_or(&"".to_string()).clone();

        Ok(s.chars().count())
    }

    fn remove_elem(&mut self, key: &str) -> Result<(), ExecError> {
        if key == "*" || key == "@" {
            self.body.clear();
            return Ok(());
        }

        if let Ok(n) = key.parse::<usize>() {
            self.body.remove(&n);
            return Ok(());
        }
        Err(ExecError::Other("invalid index".to_string()))
    }
}

impl ArrayData {
    pub fn set_new_entry(db_layer: &mut HashMap<String, Box<dyn Data>>, name: &str, v: Option<Vec<String>>)
    -> Result<(), ExecError> {
        if v.is_none() {
            db_layer.insert(name.to_string(), UninitArray{}.boxed_clone());
        }else {
            db_layer.insert(name.to_string(), Box::new(ArrayData::from(v)));
        }
        Ok(())
    }

    pub fn set_elem(db_layer: &mut HashMap<String, Box<dyn Data>>,
                        name: &str, pos: usize, val: &String) -> Result<(), ExecError> {
        if let Some(d) = db_layer.get_mut(name) {
            if d.is_array() {
                if ! d.is_initialized() {
                    *d = ArrayData::default().boxed_clone();
                }
                
                return d.set_as_array(&pos.to_string(), val);
            }else if d.is_assoc() {
                return d.set_as_assoc(&pos.to_string(), val);
            }else{
                let data = d.get_as_single()?;
                ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
                Self::set_elem(db_layer, name, 0, &data)?;
                Self::set_elem(db_layer, name, pos, val)
            }
        }else{
            ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
            Self::set_elem(db_layer, name, pos, val)
        }
    }

    pub fn append_elem(db_layer: &mut HashMap<String, Box<dyn Data>>,
                        name: &str, pos: usize, val: &String) -> Result<(), ExecError> {
        if let Some(d) = db_layer.get_mut(name) {
            if d.is_array() {
                if ! d.is_initialized() {
                    *d = ArrayData::default().boxed_clone();
                }

                return d.append_to_array_elem(&pos.to_string(), val);
            }else if d.is_assoc() {
                return d.append_to_assoc_elem(&pos.to_string(), val);
            }else{
                let data = d.get_as_single()?;
                ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
                Self::append_elem(db_layer, name, 0, &data)?;
                Self::append_elem(db_layer, name, pos, val)
            }
        }else{
            ArrayData::set_new_entry(db_layer, name, Some(vec![]))?;
            Self::set_elem(db_layer, name, pos, val)
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
