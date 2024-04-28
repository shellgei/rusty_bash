//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::feeder::terminal::Terminal;
use glob;
use glob::{GlobError, MatchOptions};
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

fn expand(path: &str) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
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
        Ok(p) => {
            let mut s = p.to_string_lossy().to_string();
            if p.is_dir() && s.chars().last() != Some('/') {
                s.push('/');
            }
            s
        },
        _ => "".to_string(),
    }
}

fn common_length(chars: &Vec<char>, s: &String) -> usize {
    let max_len = chars.len();
    for (i, c) in s.chars().enumerate() {
        if i >= max_len || chars[i] != c {
            return i;
        }
    }
    max_len
}

fn common_string(paths: &Vec<String>) -> String {
    if paths.len() == 0 {
        return "".to_string();
    }

    let ref_chars: Vec<char> = paths[0].chars().collect();
    let mut common_len = ref_chars.len();

    for path in &paths[1..] {
        let len = common_length(&ref_chars, &path);
        common_len = std::cmp::min(common_len, len);
    }

    ref_chars[..common_len].iter().collect()
}

impl Terminal {
    pub fn completion (&mut self, double_tab: bool) {
        self.file_completion(double_tab);
    }

    pub fn file_completion (&mut self, double_tab: bool) {
        let input = self.get_string(self.prompt.chars().count());
        let last = match input.split(" ").last() {
            Some(s) => s, 
            None => return, 
        };

        let mut paths = expand(&(last.to_owned() + "*"));
        match paths.len() {
            0 => { self.cloop(); },
            1 => self.replace_input(&paths[0], &last),
            _ => {
                let common = common_string(&paths);
                if common.len() == last.len() {
                    if double_tab {
                        if last.chars().last() == Some('/') {
                            let mut reduced_paths = vec![];
                            for p in paths.iter() {
                                reduced_paths.push(p.replacen(last, "", 1));
                            }
                            paths = reduced_paths;
                        }
                        self.show_path_candidates(&paths);
                    }else{
                        self.cloop();
                    }
                    return;
                }
                self.replace_input(&common, &last);
            },
        }
    }

    fn show_list(&mut self, list: &Vec<String>) {
        eprintln!("\r");

        let widths: Vec<usize> = list.iter().map(|p| UnicodeWidthStr::width(p.as_str())).collect();
        let max_entry_width = widths.iter().max().unwrap_or(&1000) + 1;

        let col_num = Terminal::size().0 / max_entry_width;
        if col_num == 0 {
            list.iter().for_each(|p| print!("{}\r\n", &p));
            self.rewrite(true);
            return;
        }

        let row_num = (list.len()-1) / col_num + 1;

        for row in 0..row_num {
            for col in 0..col_num {
                let i = col*row_num + row;
                if i >= list.len() {
                    continue;
                }

                let space_num = max_entry_width - widths[i];
                let s = String::from_utf8(vec![b' '; space_num]).unwrap();
                print!("{}{}", list[i], &s);
            }
            print!("\r\n");
        }
        self.rewrite(true);
    }

    pub fn show_path_candidates(&mut self, paths: &Vec<String>) {
        self.show_list(paths);
    }

    fn replace_input(&mut self, path: &String, last: &str) {
        let last_char_num = last.chars().count();
        let len = self.chars.len();
        let path_chars = path.to_string();

        self.chars.drain(len - last_char_num..);
        self.chars.extend(path_chars.chars());
        self.head = self.chars.len();
        self.rewrite(false);
    }
}
