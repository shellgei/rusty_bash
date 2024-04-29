//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::feeder::terminal::Terminal;
use faccess;
use faccess::PathExt;
use glob;
use glob::{GlobError, MatchOptions};
use std::collections::HashSet;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

fn expand(path: &str, executable_only: bool, search_dir: bool) -> Vec<String> {
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    let mut ans: Vec<String> = match glob::glob_with(&path, opts) {
        Ok(ps) => ps.map(|p| to_str(&p, executable_only, search_dir))
                    .filter(|s| s != "").collect(),
        _ => return vec![],
    };

    ans.sort();
    ans
}

fn to_str(path :&Result<PathBuf, GlobError>, executable_only: bool, search_dir: bool) -> String {
    match path {
        Ok(p) => {
            if ( executable_only && ! p.executable() && ! p.is_dir() )
            || ( ! search_dir && p.is_dir() ) {
                return "".to_string();
            }

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
    pub fn completion(&mut self, core: &mut ShellCore, double_tab: bool) {
        let input = self.get_string(self.prompt.chars().count());
        let words: Vec<String> = input.split(" ").map(|e| e.to_string()).collect();
        if words.len() == 0 || words.last().unwrap() == "" {
            self.cloop();
            return;
        }

        let last = words.last().unwrap().clone();

        let mut command_pos = 0;
        for w in &words {
            match w.find("=") {
                None => break,
                _    => command_pos +=1,
            }
        }
        let search_command = command_pos == words.len()-1;

        match search_command && ! last.starts_with(".") && ! last.starts_with("/"){
            true  => self.command_completion(&last, core),
            false => self.file_completion(&last, core, double_tab, search_command),
        }
    }

    pub fn command_list(target: &String, core: &mut ShellCore) -> Vec<String> {
        let mut comlist = HashSet::new();
        for path in core.get_param_ref("PATH").to_string().split(":") {
            for file in expand(&(path.to_string() + "/*"), true, false) {
                let command = file.split("/").last().map(|s| s.to_string()).unwrap();
                if command.starts_with(target) {
                    comlist.insert(command.clone());
                }
            }
        }
        let mut ans: Vec<String> = comlist.iter().map(|c| c.to_string()).collect();
        ans.sort();
        ans
    }

    pub fn command_completion(&mut self, target: &String, core: &mut ShellCore) {
        let comlist = Self::command_list(target, core);
        match comlist.len() {
            0 => self.cloop(),
            1 => self.replace_input(&(comlist[0].to_string() + " "), &target),
            _ => self.show_list(&comlist),
        }
    }

    pub fn file_completion(&mut self, target: &String, core: &mut ShellCore,
                           double_tab: bool, search_executable: bool) {
        let mut wildcard = target.to_string() + "*";

        let mut target_tilde = target.to_string();
        if target.starts_with("~/") {
            self.tilde_prefix = "~/".to_string();
            self.tilde_path = core.get_param_ref("HOME").to_string() + "/";
            wildcard = wildcard.replacen(&self.tilde_prefix, &self.tilde_path, 1);    
            target_tilde = target_tilde.replacen(&self.tilde_prefix, &self.tilde_path, 1);
        }else{
            self.tilde_prefix = String::new();
            self.tilde_path = String::new();
        }

        let paths = expand(&wildcard, search_executable, true);
        match paths.len() {
            0 => self.cloop(),
            1 => self.replace_input(&paths[0], &target),
            _ => self.file_completion_multicands(&target_tilde, &paths, double_tab),
        }
    }

    fn show_list(&mut self, list: &Vec<String>) {
        eprintln!("\r");

        let widths: Vec<usize> = list.iter()
                                     .map(|p| UnicodeWidthStr::width(p.as_str()))
                                     .collect();
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

    pub fn file_completion_multicands(&mut self, dir: &String,
                                      paths: &Vec<String>, double_tab: bool) {
        let common = common_string(&paths);
        if common.len() == dir.len() {
            match double_tab {
                true => self.show_path_candidates(&dir.to_string(), &paths),
                false => self.cloop(),
            }
            return;
        }
        self.replace_input(&common, &dir);
    }

    pub fn show_path_candidates(&mut self, dir: &String, paths: &Vec<String>) {
        let ps = if dir.chars().last() == Some('/') {
            paths.iter()
                 .map(|p| p.replacen(dir, "", 1)
                 .replacen(&self.tilde_path, &self.tilde_prefix, 1))
                 .collect()
        }else{
            paths.iter()
                 .map(|p| p.replacen(&self.tilde_path, &self.tilde_prefix, 1))
                 .collect()
        };

        self.show_list(&ps);
    }

    fn replace_input(&mut self, path: &String, last: &str) {
        let last_char_num = last.chars().count();
        let len = self.chars.len();
        let mut path_chars = path.to_string();

        if last.starts_with("./") {
            path_chars.insert(0, '/');
            path_chars.insert(0, '.');
        }
        
        path_chars = path_chars.replacen(&self.tilde_path, &self.tilde_prefix, 1);

        self.chars.drain(len - last_char_num..);
        self.chars.extend(path_chars.chars());
        self.head = self.chars.len();
        self.rewrite(false);
    }
}
