//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    pub data: Vec<String>,
}

/*
impl From<&str> for SingleData {
    fn from(s: &str) -> Self {
        Self {
            data: s.to_string(),
            ..Default::default()
        }
    }
}

impl From<&String> for SingleData {
    fn from(s: &String) -> Self {
        Self {
            data: s.clone(),
            ..Default::default()
        }
    }
}
*/

impl From<Vec<String>> for ArrayData {
    fn from(v: Vec<String>) -> Self {
        Self {
            data: v,
            ..Default::default()
        }
    }
}

impl ArrayData {
    pub fn set(&mut self, pos: usize, val: &String) -> bool {
        if self.data.len() > pos {
            self.data[pos] = val.clone();
            true
        }else{
            false
        }
    }
}
