//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;
use crate::utils::glob;

impl BracedParam {
    pub fn remove(&mut self, core: &mut ShellCore) -> bool {
       let pattern = match &self.remove_pattern {
           Some(w) => {
               match w.eval_for_case_word(core) {
                   Some(s) => s,
                   None    => return false,
               }
           },
           None => return true,
       };

       let extglob = core.shopts.query("extglob");

       if self.remove_symbol.starts_with("#") {
           let mut length = 0;
           let mut ans_length = 0;

           for ch in self.text.chars() {
               length += ch.len_utf8();
               let s = self.text[0..length].to_string();

               if glob::compare(&s, &pattern, extglob) {
                   ans_length = length;
                   if self.remove_symbol == "#" {
                       break;
                   }
               }
           }

           self.text = self.text[ans_length..].to_string();
           return true;
       }

       if self.remove_symbol.starts_with("%") {
           let mut length = self.text.len();
           let mut ans_length = length;

           for ch in self.text.chars().rev() {
               length -= ch.len_utf8();
               let s = self.text[length..].to_string();

               if glob::compare(&s, &pattern, extglob) {
                   ans_length = length;
                   if self.remove_symbol == "%" {
                       break;
                   }
               }
           }

           self.text = self.text[0..ans_length].to_string();
           return true;
       }

       true
    }
}
