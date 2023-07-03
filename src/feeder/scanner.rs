//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;
use crate::ShellCore;

impl Feeder {
    fn feed_and_connect(&mut self, core: &mut ShellCore) -> bool {
        self.remaining.pop();
        self.remaining.pop();
        return self.feed_additional_line(core);
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

        if next_line && self.feed_and_connect(core){
            return self.scanner_chars(charlist, core);
        }
        ans
    }

    pub fn scanner_word(&mut self, core: &mut ShellCore) -> usize {
        if self.remaining.starts_with("#") {
            return 0;
        }

        let mut next_line = false; 
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if &self.remaining[ans..] == "\\\n" {
                next_line = true;
                break;
            }else if let Some(_) = " \t\n;&|()".find(ch) {
                break;
            }
            ans += ch.len_utf8();
        }

        if next_line && self.feed_and_connect(core){
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

    pub fn scanner_job_end(&mut self) -> usize {
        if let Some(ch) = self.remaining.chars().nth(0) {
            if let Some(_) = ";&\n".find(ch) {
                return 1;
            }
        }
        0
    }

    pub fn scanner_pipe(&mut self, core: &mut ShellCore) -> usize {
        if self.remaining.starts_with("|\\\n"){
            if self.feed_and_connect(core){
                return self.scanner_pipe(core);
            }else{
                return 1;
            }
        }

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
}
