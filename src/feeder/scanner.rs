//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;
use crate::ShellCore;

impl Feeder {
    pub fn scanner_word(&mut self, core: &mut ShellCore) -> usize {
        if self.remaining.starts_with("#") {
            return 0;
        }
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if let Some(_) = " \t\n;&|()".find(ch) {
                break;
            }
            ans += ch.len_utf8();
        }

        if self.remaining.ends_with("\\\n")
            && self.remaining.len() == ans + 1 {
            self.remaining.pop();
            self.remaining.pop();
            if self.feed_additional_line(core){
                return self.scanner_word(core);
            }else{
                return ans - 1;
            }
        }

        ans
    }

    pub fn scanner_blank(&mut self, core: &mut ShellCore) -> usize {
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if let Some(_) = " \t".find(ch) {
                ans += 1;
            }else{
                break;
            }
        }

        if self.remaining.ends_with("\\\n")
            && self.remaining.len() == ans + 2 {
            self.remaining.pop();
            self.remaining.pop();
            if self.feed_additional_line(core){
                return self.scanner_blank(core);
            }else{
                return ans;
            }
        }
        ans
    }

    pub fn scanner_job_end(&mut self) -> usize {
        if let Some(ch) = self.remaining.chars().nth(0) {
            if let Some(_) = ";&\n".find(ch) {
                return 1;
            }
        }
        0
    }

    pub fn scanner_pipe(&self) -> usize {
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
