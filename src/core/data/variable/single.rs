//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

#[derive(Debug, Clone, Default)]
pub struct SingleData {
    pub data: String,
    pub attributes: String,
}

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

impl From<String> for SingleData {
    fn from(s: String) -> Self {
        Self {
            data: s,
            ..Default::default()
        }
    }
}
