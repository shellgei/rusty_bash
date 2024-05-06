//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::builtins::completion;
use crate::feeder::terminal::Terminal;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

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
        let (tilde_prefix, tilde_path, last_tilde_expanded) = Self::set_tilde_transform(&last, core);

        let mut command_pos = 0;
        for w in &words {
            match w.find("=") {
                None => break,
                _    => command_pos +=1,
            }
        }
        let search_command = command_pos == words.len()-1;

        let mut args = vec!["".to_string(), "".to_string(), last_tilde_expanded.to_string()];
        let list = match search_command {
            true  => completion::compgen_c(core, &mut args),
            false => completion::compgen_f(core, &mut args),
        };

        if list.len() == 0 {
            self.cloop();
            return;
        }

        let list_output = list.iter().map(|p| p.replacen(&tilde_path, &tilde_prefix, 1)).collect();
        core.data.set_array("COMPREPLY", &list_output);

        if double_tab {
            self.show_list(&core.data.arrays["COMPREPLY"]);
            return;
        }

        if list.len() == 1 {
            let output = core.data.arrays["COMPREPLY"][0].clone();
            let tail = match Path::new(&list[0]).is_dir() {
                true  => "/",
                false => " ",
            };
            self.replace_input(&(output + tail), &last);
            return;
        }

        let common = common_string(&core.data.arrays["COMPREPLY"]);
        if common.len() < last.len() {
            self.replace_input(&common, &last);
            return;
        }
        self.cloop();
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

    fn replace_input(&mut self, path: &String, last: &str) {
        let last_char_num = last.chars().count();
        let len = self.chars.len();
        let mut path_chars = path.to_string();

        if last.starts_with("./") {
            path_chars.insert(0, '/');
            path_chars.insert(0, '.');
        }
        
        self.chars.drain(len - last_char_num..);
        self.chars.extend(path_chars.chars());
        self.head = self.chars.len();
        self.rewrite(false);
    }

    fn set_tilde_transform(last: &str, core: &mut ShellCore) -> (String, String, String) {
        let tilde_prefix;
        let tilde_path;
        let last_tilde_expanded;

        if last.starts_with("~/") {
            tilde_prefix = "~/".to_string();
            tilde_path = core.data.get_param_ref("HOME").to_string() + "/";
            last_tilde_expanded = last.replacen(&tilde_prefix, &tilde_path, 1);
        }else{
            tilde_prefix = String::new();
            tilde_path = String::new();
            last_tilde_expanded = last.to_string();
        }

        (tilde_prefix, tilde_path, last_tilde_expanded)
    }
}
