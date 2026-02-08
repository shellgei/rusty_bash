//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Completion {
    pub entries: HashMap<String, CompletionEntry>,
    pub current: CompletionEntry,
    pub default_function: String,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionEntry {
    pub function: String,
    pub o_options: Vec<String>,
    pub action: String,
    pub options: HashMap<String, String>,
    pub large_w_cands: String,
}
