//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::operators::{ControlOperator, RedirectOp};

impl Feeder {
    pub fn scanner_until_escape(&mut self, to: &str) -> usize {
        let mut pos = 0;
        let mut escaped = false;
        for ch in self.chars_after(0) {
            if escaped || ch == '\\' {
                escaped = !escaped;
            }else if let Some(_) = to.find(ch) {
                break;
            };
            pos += ch.len_utf8();
        }
        pos
    }
    
    pub fn scanner_until(&mut self, from: usize, to: &str) -> usize {
        let mut pos = from;
        for ch in self.chars_after(from) {
            if let Some(_) = to.find(ch) {
                break;
            };
            pos += ch.len_utf8();
        }
        pos
    }

    pub fn scanner_name_or_parameter(&mut self) -> usize {
        let ans = self.scanner_parameter(0);
    
        if ans == 0 {
            self.scanner_name()
        }else{
            ans
        }
    }
    
    pub fn scanner_number(&mut self, from: usize) -> usize {
        let mut pos = from;
        for ch in self.chars_after(from) {
            if ch < '0' || '9' < ch {
                break;
            }
            pos += 1;
        }
        pos
    }

    pub fn scanner_non_quoted_word(&mut self, in_brace: bool, ignore_brace: bool) -> usize {
        let mut escaped = false;
        let mut pos = 0;
        for ch in self.remaining.chars() {
            if escaped {
                escaped = false;
                pos += ch.len_utf8();
                continue;
            }

            if ch == '\\' {
                escaped = true;
                pos += ch.len_utf8();
                continue;
            }

            /* stop at meta characters, \n, quotes, start of brace, start of expansion*/
            if let Some(_) = "|&;()<> \t\n\"'$".find(ch) {
                break;
            }
            if ! ignore_brace && ch == '{' {
                break;
            }
            if in_brace && ( ch == ',' || ch == '}') {
                break;
            }

            pos += ch.len_utf8();
        }

        pos
    }

    pub fn scanner_double_quoted_word(&mut self) -> usize {
        let mut escaped = false;
        let mut pos = 0;
        for ch in self.remaining.chars() {
            if escaped {
                escaped = false;
                pos += ch.len_utf8();
                continue;
            }

            if ch == '\\' {
                escaped = true;
                pos += ch.len_utf8();
                continue;
            }

            /* stop at double quote or $*/
            if let Some(_) = "\"$".find(ch) {
                break;
            }

            pos += ch.len_utf8();
        }

        pos
    }

    pub fn scanner_redirect(&mut self) -> (usize, Option<RedirectOp> ) {
        if self.starts_with("<<<") {
            return (3, Some(RedirectOp::HereStr));
        }else if self.starts_with("&>>") {
            return (3, Some(RedirectOp::AndAppend));
        }else if self.starts_with(">>") {
            return (2, Some(RedirectOp::Append));
        }else if self.starts_with("<<") {
            return (2, Some(RedirectOp::HereDoc));
        }else if self.starts_with(">&") {
            return (2, Some(RedirectOp::OutputAnd));
        }else if self.starts_with("&>") {
            return (2, Some(RedirectOp::AndOutput));
        }else if self.starts_with("<>") {
            return (2, Some(RedirectOp::InOut));
        }else if self.starts_with(">") {
            return (1, Some(RedirectOp::Output));
        }else if self.starts_with("<") {
            return (1, Some(RedirectOp::Input));
        }
        (0, None)
    }

    pub fn scanner_name(&mut self) -> usize {
        if self.len() <= 0 {
            return 0;
        }
    
        let h = &self.nth(0);
        if !((*h >= 'A' && *h <= 'Z') || (*h >= 'a' && *h <= 'z') || *h == '_') {
            return 0;
        }
    
        if self.len() == 1 {
            return 1;
        }
    
        let mut ans = 1;
        for c in self.chars_after(1) {
            if !((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') 
            || (c >= 'a' && c <= 'z') || c == '_') {
                break;
            }
            ans += 1;
        }
    
        return ans;
    }

    pub fn scanner_tilde_prefix(&mut self) -> usize {
        if ! self.starts_with("~") {
            return 0;
        }

        return self.scanner_until(0, " \n\t\"';{}()$<>&:/,");
    }

    pub fn scanner_control_op(&mut self) -> (usize, Option<ControlOperator> ) {
        let mut op = None;
        let mut pos = 0;
    
        if self.len() > 2  {
            pos = 3;
            op = if self.starts_with(";;&") {
                Some(ControlOperator::SemiSemiAnd)
            }else{
                None
            };
        }
    
        if op == None && self.len() > 1  {
            pos = 2;
            op = if self.starts_with("||") {
                Some(ControlOperator::Or)
            }else if self.starts_with("&&") {
                Some(ControlOperator::And)
            }else if self.starts_with(";;") {
                Some(ControlOperator::DoubleSemicolon)
            }else if self.starts_with(";&") {
                Some(ControlOperator::SemiAnd)
            }else if self.starts_with("|&") {
                Some(ControlOperator::PipeAnd)
            }else{
                None
            };
    
        }
    
        if op == None && self.len() > 0  {
            pos = 1;
            if self.starts_with("&") {
                if self.len() > 1 && self.nth(1) == '>' {
                    return (0, None)
                }
                return (1, Some(ControlOperator::BgAnd));
            } else if self.starts_with("\n") {
                return (1, Some(ControlOperator::NewLine));
            } else if self.starts_with("|") {
                return (1, Some(ControlOperator::Pipe));
            } else if self.starts_with(";") {
                return (1, Some(ControlOperator::Semicolon));
            } else if self.starts_with("(") {
                return (1, Some(ControlOperator::LeftParen));
            } else if self.starts_with(")") {
                return (1, Some(ControlOperator::RightParen));
            }
        }
    
        if op != None && self.len() > pos && self.nth(pos) == '\n' {
            pos += 1;
        }
    
        if op != None{
            return (pos, op);
        }
    
    
        (0 , None)
    }

    pub fn scanner_comment(&mut self) -> usize {
        if self.starts_with("#") {
            return self.scanner_until(0, "\n");
        }
    
        0
    }

    pub fn scanner_integer(&mut self) -> usize {
        if self.len() == 0 {
            return 0;
        }
    
        let mut pos = 0;
        let mut minus = false;
        if self.starts_with("-") {
            pos += 1;
            minus = true;
        }
    
        for ch in self.chars_after(pos) {
            if ch < '0' || ch > '9' {
                break;
            }
    
            pos += 1;
        }
    
        if minus && pos == 1 {
            0
        }else{
            pos
        }
    }

    fn scanner_parameter(&mut self, from: usize) -> usize {
        if self.len() < from {
            return from;
        }
    
        if "?*@$#!-:".chars().any(|c| c == self.nth(from)) { //special parameters
            return from+1;
        };
    
        self.scanner_number(from)
    }
    
}
