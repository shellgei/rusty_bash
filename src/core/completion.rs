//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Completion {
    pub entries: HashMap<String, CompletionSpec>,
    pub current: CompletionSpec,
    pub default_spec: Option<CompletionSpec>,
    pub empty_spec: Option<CompletionSpec>,
    pub initial_spec: Option<CompletionSpec>,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionSpec {
    pub actions: Vec<String>,
    pub glob_pattern: Option<String>,
    pub wordlist: Option<String>,
    pub function: Option<String>,
    pub command: Option<String>,
    pub filter_pattern: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub options: Vec<String>,
}

impl CompletionSpec {
    pub fn has_option(&self, option: &str) -> bool {
        self.options.iter().any(|known| known == option)
    }

    pub fn filenames(&self) -> bool {
        self.has_option("filenames")
            || self.has_option("fullquote")
            || self
                .actions
                .iter()
                .any(|action| matches!(action.as_str(), "file" | "directory"))
    }

    pub fn allows_default_completion(&self) -> bool {
        ["bashdefault", "default", "dirnames", "plusdirs"]
            .iter()
            .any(|option| self.has_option(option))
    }
}
