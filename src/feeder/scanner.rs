//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;

impl Feeder {
    pub fn scanner_word(&mut self) -> usize {
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if let Some(_) = " \t\n;&|".find(ch) {
                break;
            }
            ans += ch.len_utf8();
        }
        ans
    }

    pub fn scanner_blank(&mut self) -> usize {
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if let Some(_) = " \t".find(ch) {
                ans += 1;
            }else{
                break;
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
}
