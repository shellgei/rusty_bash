//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::feeder::terminal::Terminal;
use glob;
use glob::{GlobError, MatchOptions};
use std::path::PathBuf;

fn expand(path: &str) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: true,
    };

    let mut ans: Vec<String> = match glob::glob_with(&path, opts) {
        Ok(ps) => ps.map(|p| to_str(&p))
                    .filter(|s| s != "").collect(),
        _ => return vec![],
    };

    ans.sort();
    ans
}

fn to_str(path :&Result<PathBuf, GlobError>) -> String {
    match path {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => "".to_string(),
    }
}

impl Terminal {
    pub fn completion (&mut self, core: &mut ShellCore) {
        let input = self.get_string(self.prompt.chars().count());
        let last = match input.split(" ").last() {
            Some(s) => s, 
            None => return, 
        };

        let paths = expand(&(last.to_owned() + "*"));
        match paths.len() {
            1 => self.replace_input(&paths[0], &last),
            _ => {},
        }
    }

    fn replace_input(&mut self, path: &String, last: &str) {
        let last_char_num = last.chars().count();
        let len = self.chars.len();
        let path_chars = path.to_string();

        self.chars.drain(len - last_char_num..);
        self.chars.extend(path_chars.chars());
        self.head = self.chars.len();
        self.rewrite(false)
    }
}

