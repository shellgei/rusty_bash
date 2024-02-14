//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;
use crate::ShellCore;

impl Feeder {
    fn feed_and_connect(&mut self, core: &mut ShellCore) {
        self.remaining.pop();
        self.remaining.pop();
        let _ = self.feed_additional_line_core(core);
    }

    fn backslash_check_and_feed(&mut self, starts: Vec<&str>, core: &mut ShellCore) {
        let check = |s: &str| self.remaining.starts_with(&(s.to_owned() + "\\\n"));
        if starts.iter().any(|s| check(s)) {
            self.feed_and_connect(core);
        }
    }

    fn scanner_chars(&mut self, charlist: &str, core: &mut ShellCore) -> usize {
        let mut next_line = false;
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if &self.remaining[ans..] == "\\\n" {
                next_line = true;
                break;
            }else if charlist.find(ch) != None {
                ans += 1;
            }else{
                break;
            }
        }

        if next_line {
            self.feed_and_connect(core);
            return self.scanner_chars(charlist, core);
        }
        ans
    }

    pub fn scanner_subword_symbol(&self) -> usize {
        if self.starts_with("{")
        || self.starts_with(",")
        || self.starts_with("}")
        || self.starts_with("$"){
            1
        }else{
            0
        }
    }

    pub fn scanner_escaped_char(&mut self, core: &mut ShellCore) -> usize {
        if self.starts_with("\\\n") {
            self.feed_and_connect(core);
        }

        if ! self.starts_with("\\") {
            return 0;
        }

        match self.remaining.chars().nth(1) {
            Some(ch) => 1 + ch.len_utf8(),
            None =>     1,
        }
    }

    pub fn scanner_special_param(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("$") {
            return 0;
        }
        self.backslash_check_and_feed(vec!["$"], core);

        let ch = self.remaining.chars().nth(1).unwrap();
        if "$?@#-!0_".find(ch) != None {
            2
        }else{
            0
        }
    }

    pub fn scanner_subword(&mut self) -> usize {
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if " \t\n;&|()<>{},\\'".find(ch) != None {
                break;
            }
            ans += ch.len_utf8();
        }
        ans
    }

    pub fn scanner_single_quoted_subword(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("'") {
            return 0;
        }

        loop {
            if let Some(n) = self.remaining[1..].find("'") {
                return n + 2;
            }else if ! self.feed_additional_line(core) {
                break;
            }
        }
        0
    }

    pub fn scanner_blank(&mut self, core: &mut ShellCore) -> usize {
        self.scanner_chars(" \t", core)
    }

    pub fn scanner_multiline_blank(&mut self, core: &mut ShellCore) -> usize {
        self.scanner_chars(" \t\n", core)
    }

    pub fn scanner_nonnegative_integer(&mut self, core: &mut ShellCore) -> usize {
        self.scanner_chars("0123456789", core)
    }

    pub fn scanner_job_end(&mut self) -> usize {
        if let Some(ch) = self.remaining.chars().nth(0) {
            if ";&\n".find(ch) != None {
                return 1;
            }
        }
        0
    }

    pub fn scanner_and_or(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["|", "&"], core);

        if self.remaining.starts_with("||")     { 2 }
        else if self.remaining.starts_with("&&"){ 2 }
        else{ 0 }
    }

    pub fn scanner_pipe(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["|"], core);

        if self.remaining.starts_with("||")     { 0 }
        else if self.remaining.starts_with("|&"){ 2 }
        else if self.remaining.starts_with("|") { 1 }
        else{ 0 }
    }

    pub fn scanner_comment(&self) -> usize {
        if ! self.remaining.starts_with("#") {
            return 0;
        }

        let mut ans = 0;
        for ch in self.remaining.chars() {
            if "\n".find(ch) != None {
                break;
            }
            ans += ch.len_utf8();
        }
        ans
    }

    pub fn scanner_redirect_symbol(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec![">", "&"], core);

        if self.remaining.starts_with("&>")     { 2 } //追加
        else if self.remaining.starts_with(">>"){ 2 }
        else if self.remaining.starts_with("<"){ 1 }
        else if self.remaining.starts_with(">"){ 1 }
        else{ 0 }
    }
}
