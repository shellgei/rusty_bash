//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use super::super::{
    ArrayData, AssocData, IntArrayData, IntAssocData, IntData, SingleData
};

#[derive(Debug, Clone)]
pub struct Uninit {
    flags: String,
}

impl Data for Uninit {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
    fn _get_fmt_string(&self) -> String {
        "".to_string()
    }

    fn initialize(&mut self) -> Option<Box<dyn Data>> {
        let num = self.has_flag('i');

        if self.has_flag('a') {
            match num {
                true => {
                    let mut d = IntArrayData::new();
                    d.flags = self.flags.clone();
                    return Some(Box::new(d));
                },
                false => {
                    let mut d = ArrayData::new();
                    d.flags = self.flags.clone();
                    return Some(Box::new(d));
                },
            }
        }

        if self.has_flag('A') {
            match num {
                true => {
                    let mut d = IntAssocData::new();
                    d.flags = self.flags.clone();
                    return Some(Box::new(d));
                },
                false => {
                    let mut d = AssocData::new();
                    d.flags = self.flags.clone();
                    return Some(Box::new(d));
                },
            }
        }

        match num {
            true => {
                let mut d = IntData::new();
                d.flags = self.flags.clone();
                Some(Box::new(d))
            },
            false => {
                let d = SingleData::new(&self.flags);
                Some(Box::new(d))
            },
        }
    }

    fn clear(&mut self) {}
    fn is_initialized(&self) -> bool {
        false
    }
    fn get_as_array(&mut self, _: &str, _: &str) -> Result<String, ExecError> {
        Ok("".to_string())
    }
    fn get_all_as_array(&mut self, _: bool) -> Result<Vec<String>, ExecError> {
        Ok(vec![])
    }
    fn get_all_indexes_as_array(&mut self) -> Result<Vec<String>, ExecError> {
        Ok(vec![])
    }
    fn get_as_single(&mut self) -> Result<String, ExecError> {
        Ok("".to_string())
    }
    fn len(&mut self) -> usize {
        0
    }
    fn elem_len(&mut self, _: &str) -> Result<usize, ExecError> {
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

    fn is_assoc(&self) -> bool {
        self.flags.contains('A')
    }

    fn is_array(&self) -> bool {
        self.flags.contains('a')
    }

    fn get_flags(&mut self) -> String {
        self.flags.clone()
    }
}

impl Uninit {
    pub fn new(flags: &str) -> Self {
        Self { flags: flags.to_string() }
    }
}
