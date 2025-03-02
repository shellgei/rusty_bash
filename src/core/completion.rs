//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct CompletionInfo {
    pub function: String,
    pub o_options: Vec<String>,
    pub action: String,
    pub options: HashMap<String, String>,
}

/*
impl CompletionInfo {
    pub fn new(function: &String) -> Self {
        Self {
            function: function.clone(),
            ..Default::default()
        }
    }
}*/
