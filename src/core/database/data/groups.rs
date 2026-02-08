//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
//use nix::unistd;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Groups {
//    pub body: HashMap<usize, String>,
    pub flags: String,
}

impl Data for Groups {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn _get_fmt_string(&self) -> String {
        return "*********".to_string()
    }

    fn get_fmt_string(&mut self) -> String {
        let mut formatted = "(".to_string();
        let vs = self.values();
        for i in self.keys() {
            let ansi = utils::to_ansi_c(&vs[i]);
            if ansi == vs[i] {
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

    /*
    fn clear(&mut self) {
        self.body.clear();
    }*/

    fn set_as_single(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Ok(())
    }

    fn append_as_single(&mut self, _: &str, _: &str) -> Result<(), ExecError> {
        Ok(())
    }

    fn set_as_array(&mut self, _: &str, _: &str, _: &str)
                    -> Result<(), ExecError> {
        Err(ExecError::Silent)
    }

    fn append_to_array_elem(&mut self, _: &str, _: &str,
                            _: &str) -> Result<(), ExecError> {
        Err(ExecError::Silent)
    }

    fn get_as_array(&mut self, key: &str, ifs: &str) -> Result<String, ExecError> {
        if key == "@" {
            return Ok(self.values().join(" "));
        }
        if key == "*" {
            return Ok(self.values().join(ifs));
        }

        /*
        let n = self.index_of(key)?;
        Ok(self.body.get(&n).unwrap_or(&"".to_string()).clone())
        */
        Ok("".to_string())
    }

    fn get_vec_from(&mut self, _: usize, _: bool) -> Result<Vec<String>, ExecError> {
        /*
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
        */
        Ok(vec![])
    }

    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(self.keys().iter().map(|k| k.to_string()).collect())
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        /*
        self.body
            .get(&0)
            .map(|v| Ok(v.clone()))
            .ok_or(ExecError::Other("No entry".to_string()))?
        */
        let vs = self.values();
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
        //self.body.len()
        0
    }

    fn has_key(&mut self, key: &str) -> Result<bool, ExecError> {
        if key == "@" || key == "*" {
            return Ok(true);
        }
        /*
        let n = self.index_of(key)?;
        Ok(self.body.contains_key(&n))
        */
        Ok(false)
    }

    fn index_based_len(&mut self) -> usize {
        /*
        match self.body.iter().map(|e| e.0).max() {
            Some(n) => *n + 1,
            None => 0,
        }*/
        0
    }

    fn elem_len(&mut self, _: &str) -> Result<usize, ExecError> {
        /*
        if key == "@" || key == "*" {
            return Ok(self.len());
        }

        let n = self.index_of(key)?;
        let s = self.body.get(&n).unwrap_or(&"".to_string()).clone();

        Ok(s.chars().count())
        */
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

impl Groups {
    pub fn new() -> Self {
        Self {
            //body: HashMap::new(),
            flags: "a".to_string(),
        }
    }

    pub fn values(&self) -> Vec<String> {
//        unistd::getgroups();

        vec![]
    }

    pub fn keys(&self) -> Vec<usize> {
        let mut keys: Vec<usize> = vec![];
        keys.sort();
        keys
    }

    /*
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
    */
}
