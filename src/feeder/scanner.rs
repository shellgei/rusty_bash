//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Feeder;
use crate::ShellCore;

impl Feeder {
    fn feed_and_connect(&mut self, core: &mut ShellCore) {
        self.remaining.pop();
        self.remaining.pop();
        self.lineno += 1;
        let _ = core.db.set_param("LINENO", &self.lineno.to_string(), None);
        let _ = self.feed_additional_line_core(core);
    }

    fn backslash_check_and_feed(&mut self, starts: Vec<&str>, core: &mut ShellCore) {
        let check = |s: &str| self.remaining.starts_with(&(s.to_owned() + "\\\n"));
        if starts.iter().any(|s| check(s)) {
            self.feed_and_connect(core);
        }
    }

    pub fn scanner_char(&mut self) -> usize {
        match self.remaining.chars().next() {
            Some(c) => c.len_utf8(),
            None    => 0,
        }
    }

    fn scanner_chars(&mut self, judge: fn(char) -> bool,
                     core: &mut ShellCore, skip_bytes: usize) -> usize {
        loop {
            let mut ans = 0;
            for ch in self.remaining[skip_bytes..].chars() {
                match judge(ch) {
                    true  => ans += ch.len_utf8(),
                    false => break,
                }
            }

            match &self.remaining[skip_bytes+ans..] == "\\\n" {
                true  => self.feed_and_connect(core),
                false => return ans,
            }
        }
    }

    fn scanner_one_of(&self, cands: &[&str]) -> usize {
        for c in cands {
            if self.starts_with(c) {
                return c.len();
            }
        }
        0
    }

    pub fn scanner_subword_symbol(&self) -> usize {
        self.scanner_one_of(&["{", "}", ",", "$", "~", "/", "*", "?", "%", "!",
                              "@", "!", "+", "-", ".", ":", "=", "^", ",", "]"])
    }

    pub fn scanner_math_symbol(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec![""], core);
        self.scanner_one_of(&["/", "*", "?", ":", "+", "-", "=", "^", "%", ","])
    }

    pub fn scanner_unary_operator(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["+", "-", "!", "~"], core);
        if let Some('=') = self.remaining.chars().nth(1) {
            return 0;
        }

        self.scanner_one_of(&["+", "-", "!", "~"])
    }

    pub fn scanner_math_output_format(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["[#", "["], core);
        if ! self.starts_with("[#") {
            return 0;
        }

        let mut ans = 2;
        let mut ok = false;
        for (i, ch) in self.remaining[2..].chars().enumerate() {
            if i == 0 && ch == '#' {
                ans += 1;
                continue;
            }
            if '0' <= ch && ch <= '9' {
                ok = true;
                ans += 1;
                continue;
            }

            if ch == ']' && ok {
                return ans + 1;
            }

            break;
        }
        0
    }

    pub fn scanner_extglob_head(&self) -> usize {
        self.scanner_one_of(&["?(", "*(", "+(", "@(", "!("])
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

    pub fn scanner_ansi_c_oct(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("\\") {
            return 0;
        }

        let judge = |ch| '0' <= ch && ch <= '7';
        self.scanner_chars(judge, core, 1) + 1
    }

    pub fn scanner_ansi_c_hex(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("\\x") {
            return 0;
        }

        let mut skip = 2;
        if self.starts_with("\\x{") {
            skip = 3;
        }

        let judge = |ch| ('0' <= ch && ch <= '9') 
                         || ('a' <= ch && ch <= 'f') 
                         || ('A' <= ch && ch <= 'F'); 
        let len = self.scanner_chars(judge, core, skip) + skip;

        if skip == 3 {
            len + self.scanner_chars(|c| c == '}', core, len)
        }else{
            len
        }
    }

    pub fn scanner_ansi_unicode4(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("\\u") {
            return 0;
        }

        let judge = |ch| ('0' <= ch && ch <= '9') 
                         || ('a' <= ch && ch <= 'f') 
                         || ('A' <= ch && ch <= 'F'); 
        self.scanner_chars(judge, core, 2) + 2
    }

    pub fn scanner_ansi_unicode8(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("\\U") {
            return 0;
        }

        let judge = |ch| ('0' <= ch && ch <= '9') 
                         || ('a' <= ch && ch <= 'f') 
                         || ('A' <= ch && ch <= 'F'); 
        self.scanner_chars(judge, core, 2) + 2
    }

    pub fn scanner_history_expansion(&mut self, _: &mut ShellCore) -> usize {
        match self.starts_with("!$") {
            true  => 2,
            false => 0,
        }
    }

    pub fn scanner_dollar_special_and_positional_param(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("$") {
            return 0;
        }
        self.backslash_check_and_feed(vec!["$"], core);

        match self.remaining.chars().nth(1) {
            Some(c) => if "$?*@#-!0123456789".find(c) != None { 2 }else{ 0 },
            None    => 0,
        }
    }

    pub fn scanner_special_and_positional_param(&mut self) -> usize {
        match self.remaining.chars().nth(0) {
            Some(c) => if "$?*@#-!_0123456789".find(c) != None { 1 }else{ 0 },
            None    => 0,
        }
    }

    pub fn scanner_subword(&mut self) -> usize {
        let mut ans = 0;
        for ch in self.remaining.chars() {
            if " \t\n;&|()<>{},\\'$/~\"*+-?@!.:=^]`%".find(ch) != None {
                break;
            }
            ans += ch.len_utf8();
        }
        ans
    }

    pub fn scanner_double_quoted_subword(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| "`\"\\$".find(ch) == None;
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_extglob_subword(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| ")|,}".find(ch) == None;
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_single_quoted_subword(&mut self, core: &mut ShellCore) -> usize {
        if ! self.starts_with("'") {
            return 0;
        }

        loop {
            if let Some(n) = self.remaining[1..].find("'") {
                return n + 2;
            }else if ! self.feed_additional_line(core).is_ok() {
                return 0;
            }
        }
    }

    pub fn scanner_inner_subscript(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| "]".find(ch) == None;
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_unknown_in_param_brace(&mut self) -> usize {
        match self.remaining.chars().nth(0) {
            Some(c) => if "'$".find(c) == None { c.len_utf8() }else{ 0 },
            None    => 0,
        }
    }

    pub fn scanner_blank(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| " \t".find(ch) != None;
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_multiline_blank(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| " \t\n".find(ch) != None;
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_binary_operator(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["<<", ">>", "+", "-", "/", "*", "%", "<",
                                           ">", "=", "&", "|", "^", "/", "%"], core);
        self.scanner_one_of(&["<<=", ">>=",
            "&&", "||", "**", "==", "!=", "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=",
            ">>", "<<", "<=", ">=", "&", "^", "=", "+", "-", "/", "*", "%", "<", ">", "|", "^", ","])
    }

    pub fn scanner_substitution(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["*", "/", "%", "+", "-", "<",
                                           "<<", ">", ">>", "^", "|"], core);
        self.scanner_one_of(&["=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|="])

    }

    pub fn scanner_uint(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| '0' <= ch && ch <= '9';
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_arith_number(&mut self, core: &mut ShellCore) -> usize {
        let judge = |ch| ('0' <= ch && ch <= '9') 
                         || ('a' <= ch && ch <= 'z') 
                         || ('A' <= ch && ch <= 'Z') 
                         || ".#xX_@".contains(ch);
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_name(&mut self, core: &mut ShellCore) -> usize {
        let c = self.remaining.chars().nth(0).unwrap_or('0');
        if '0' <= c && c <= '9' {
            return 0;
        }

        let judge = |ch| ch == '_' || ('0' <= ch && ch <= '9')
                         || ('a' <= ch && ch <= 'z')
                         || ('A' <= ch && ch <= 'Z');
        self.scanner_chars(judge, core, 0)
    }

    pub fn scanner_name_and_equal(&mut self, core: &mut ShellCore) -> usize {
        let name_len = self.scanner_name(core);
        if name_len == 0 {
            return 0;
        }

        if self.remaining[name_len..].starts_with("=") {
            name_len + 1
        } else if self.remaining[name_len..].starts_with("+=") {
            name_len + 2
        }else{
            0
        }
    }

    pub fn scanner_job_end(&mut self) -> usize {
        self.scanner_one_of(&[";", "&", "\n"])
    }

    pub fn scanner_and_or(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["|", "&"], core);
        self.scanner_one_of(&["||", "&&"])
    }

    pub fn scanner_pipe(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["|"], core);
        if self.starts_with("||") {
            return 0;
        }
        self.scanner_one_of(&["|&","|"])
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
        self.backslash_check_and_feed(vec!["<<", ">", "&", "<"], core);
        self.scanner_one_of(&["<<<", "<<-", "&>", ">&", ">>", "<<", "<", ">"])
    }

    pub fn scanner_parameter_alternative_symbol(&mut self) -> usize {
        self.scanner_one_of(&[":-", ":=", ":?", ":+", "-", "+"])
    }

    pub fn scanner_parameter_remove_symbol(&mut self) -> usize {
        self.scanner_one_of(&["##", "#", "%%", "%"])
    }

    pub fn scanner_tabs(&mut self) -> usize {
        self.scanner_one_of(&["\t"])
    }

    pub fn scanner_test_check_option(&mut self, core: &mut ShellCore) -> usize {
        match self.remaining.chars().nth(0) {
            Some('-') => {},
            _ => return 0,
        }
        self.backslash_check_and_feed(vec!["-"], core);

        if let Some(c) = self.remaining.chars().nth(1) {
            match "abcdefghknoprstuvwxzGLNOS".contains(c) {
                true  => return 2,
                false => return 0,
            }
        }
        return 0;
    }

    pub fn scanner_test_compare_op(&mut self, core: &mut ShellCore) -> usize {
        self.backslash_check_and_feed(vec!["-", "-e", "-n", "-o", "=", "!"], core);
        self.scanner_one_of(&["-ef", "-nt", "-ot", "==", "=~", "=", "!=", "<", ">",
                              "-eq", "-ne", "-lt", "-le", "-gt", "-ge"])
    }

    pub fn scanner_regex_symbol(&mut self) -> usize {
        match self.starts_with(" ") {
            true  => 0,
            false => 1,
        }
    }
}
