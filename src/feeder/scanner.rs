//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;
use crate::ShellCore;

impl Feeder {
    fn feed_and_connect(&mut self, core: &mut ShellCore) {
        self.remaining.pop();
        self.remaining.pop();
        if ! self.feed_additional_line(core){
            self.remaining = String::new();
        }
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
            }else if let Some(_) = charlist.find(ch) {
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

    pub fn scanner_word(&mut self, core: &mut ShellCore) -> usize {
        let mut next_line = false; 
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if &self.remaining[ans..] == "\\\n" {
                next_line = true;
                break;
            }else if let Some(_) = " \t\n;&|()<>".find(ch) {
                break;
            }
            ans += ch.len_utf8();
        }

        if next_line {
            self.feed_and_connect(core);
            return self.scanner_word(core);
        }
        ans
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
            if let Some(_) = ";&\n".find(ch) {
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
            if let Some(_) = "\n".find(ch) {
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
