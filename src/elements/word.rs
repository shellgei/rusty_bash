//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod brace_expansion;
pub mod path_expansion;
mod split;
pub mod substitution;
pub mod tilde_expansion;

use super::subword::Subword;
use crate::elements::subword;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{utils, Feeder, ShellCore};

#[derive(Debug, Clone)]
pub enum WordMode {
    Alias,
    AlterWord,
    Arithmetic,
    AssocIndex,
    EvalLet,
    CompgenF,
    ReadCommand,
    Heredoc,
    RightOfSubstitution,
    Value,
    PermitAnyChar,
    //ReparseOfSubstitution,
    ParamOption(Vec<String>),
}

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub text: String,
    pub do_not_erase: bool,
    pub subwords: Vec<Box<dyn Subword>>,
    pub mode: Option<WordMode>,
}

impl From<&str> for Word {
    fn from(s: &str) -> Self {
        Self {
            text: s.to_string(),
            subwords: vec![From::from(s)],
            ..Default::default()
        }
    }
}

impl From<Box<dyn Subword>> for Word {
    fn from(subword: Box<dyn Subword>) -> Self {
        Self {
            text: subword.get_text().to_string(),
            subwords: vec![subword],
            ..Default::default()
        }
    }
}

impl From<Vec<Box<dyn Subword>>> for Word {
    fn from(subwords: Vec<Box<dyn Subword>>) -> Self {
        Self {
            text: subwords.iter().map(|s| s.get_text()).collect(),
            subwords,
            ..Default::default()
        }
    }
}

impl Word {
    pub fn eval(&mut self, core: &mut ShellCore) -> Result<Vec<String>, ExecError> {
        let ws_after_brace_exp = match core.db.flags.contains('B') {
            true => brace_expansion::eval(&mut self.clone(), core.compat_bash),
            false => vec![self.clone()],
        };

        let mut ws = vec![];
        for w in ws_after_brace_exp {
            let expanded = w.tilde_and_dollar_expansion(core)?;
            ws.append(&mut expanded.split_and_path_expansion(core));
        }
        Ok(Self::make_args(&mut ws))
    }

    pub fn eval_as_herestring(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        self.eval_as_value(core)
    }

    pub fn eval_as_value(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let w = self.tilde_and_dollar_expansion(core)?;
        let mut ws = w.path_expansion(core);
        let joint = core.db.get_ifs_head();
        Ok(Self::make_args(&mut ws).join(&joint))
    }

    pub fn eval_as_alter(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let w = self.dollar_expansion(core)?;
        let mut ws = w.path_expansion(core);
        let joint = core.db.get_ifs_head();
        Ok(Self::make_args(&mut ws).join(&joint))
    }

    pub fn eval_as_assoc_index(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let w = self.tilde_and_dollar_expansion(core)?;
        let joint = core.db.get_ifs_head();
        Ok(Self::make_args(&mut [w]).join(&joint))
    }

    pub fn eval_as_integer(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        utils::string_to_calculated_string(&self.text, core)
    }

    pub fn eval_for_case_word(&self, core: &mut ShellCore) -> Option<String> {
        match self.tilde_and_dollar_expansion(core) {
            Ok(mut w) => w.make_unquoted_word(),
            Err(e) => {
                e.print(core);
                None
            }
        }
    }

    pub fn eval_for_regex(&self, core: &mut ShellCore) -> Option<String> {
        let quoted = self.text.starts_with("\"") && self.text.ends_with("\"");

        match self.tilde_and_dollar_expansion(core) {
            Ok(mut w) => {
                let mut re = w.make_regex()?;
                if quoted {
                    re.insert(0, '"');
                    re += "\"";
                }

                Some(re)
            }
            Err(e) => {
                e.print(core);
                None
            }
        }
    }

    pub fn eval_for_case_pattern(&self, core: &mut ShellCore) -> Result<String, ExecError> {
        let mut w = self.tilde_and_dollar_expansion(core)?;
        Ok(w.make_glob_string())
    }

    pub fn set_pipe(&mut self, core: &mut ShellCore) {
        for sw in self.subwords.iter_mut() {
            sw.set_pipe(core);
        }
    }

    pub fn dollar_expansion(&self, core: &mut ShellCore) -> Result<Word, ExecError> {
        let mut w = self.clone();
        substitution::eval(&mut w, core)?;
        Ok(w)
    }

    pub fn tilde_and_dollar_expansion(&self, core: &mut ShellCore) -> Result<Word, ExecError> {
        let mut w = self.clone();
        tilde_expansion::eval(&mut w, core);
        substitution::eval(&mut w, core)?;
        Ok(w)
    }

    pub fn split_and_path_expansion(&self, core: &mut ShellCore) -> Vec<Word> {
        let mut ans = vec![];
        let mut splitted = split::eval(self, core);

        let len = splitted.len();
        if len > 0 {
            splitted[len - 1].do_not_erase = false;
        }

        if core.options.query("noglob") {
            return splitted;
        }

        for mut w in splitted {
            ans.append(&mut path_expansion::eval(&mut w, &core.shopts));
        }
        ans
    }

    fn path_expansion(&self, core: &mut ShellCore) -> Vec<Word> {
        if core.options.query("noglob") {
            return vec![self.clone()];
        }

        path_expansion::eval(&mut self.clone(), &core.shopts)
    }

    fn make_args(words: &mut [Word]) -> Vec<String> {
        words
            .iter_mut()
            .filter_map(|w| w.make_unquoted_word())
            .collect()
    }

    pub fn make_unquoted_word(&mut self) -> Option<String> {
        let sw: Vec<Option<String>> = self
            .subwords
            .iter_mut()
            .map(|s| s.make_unquoted_string())
            .filter(|s| s.is_some())
            .collect();

        if sw.is_empty() && !self.do_not_erase {
            return None;
        }

        Some(sw.into_iter().map(|s| s.unwrap()).collect::<String>())
    }

    pub fn make_regex(&mut self) -> Option<String> {
        let sw: Vec<Option<String>> = self
            .subwords
            .iter_mut()
            .map(|s| s.make_regex())
            .filter(|s| s.is_some())
            .collect();

        if sw.is_empty() {
            return None;
        }

        Some(sw.into_iter().map(|s| s.unwrap()).collect::<String>())
    }

    fn make_glob_string(&mut self) -> String {
        self.subwords
            .iter_mut()
            .map(|s| s.make_glob_string())
            .collect::<Vec<String>>()
            .concat()
    }

    pub fn set_heredoc_flag(&mut self) {
        self.subwords.iter_mut().for_each(|e| e.set_heredoc_flag());
    }

    pub fn is_to_proc_sub(&mut self) -> bool {
        if self.subwords.len() == 1 {
            return self.subwords[0].is_to_proc_sub();
        }

        false
    }

    fn scan_pos(&self, s: &str) -> Vec<usize> {
        self.subwords
            .iter()
            .enumerate()
            .filter(|e| e.1.get_text() == s)
            .map(|e| e.0)
            .collect()
    }

    fn push(&mut self, subword: Box<dyn Subword>) {
        self.text += subword.get_text();
        self.subwords.push(subword);
    }

    fn pre_check(feeder: &mut Feeder, mode: &Option<WordMode>) -> bool {
        if feeder.starts_with("#") && mode.is_none() || feeder.is_empty() {
            return false;
        }

        match mode {
            Some(WordMode::Arithmetic) 
            | Some(WordMode::AlterWord) 
            | Some(WordMode::CompgenF) => {
                if feeder.starts_with("}") {
                    return false;
                }
            }
            Some(WordMode::ParamOption(v)) => {
                if feeder.starts_withs(v) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    fn post_check(feeder: &mut Feeder, core: &mut ShellCore, mode: &Option<WordMode>) -> bool {
        if feeder.is_empty() {
            return false;
        }

        match mode {
            Some(WordMode::Arithmetic) | Some(WordMode::CompgenF) => {
                if feeder.starts_withs(&["]", "}"]) || feeder.scanner_math_symbol(core) != 0 {
                    return false;
                }
            },
            Some(WordMode::AlterWord) => {
                if feeder.starts_with("}") {
                    return false;
                }
            },
            Some(WordMode::ParamOption(v)) => {
                if feeder.starts_withs(v) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    pub fn parse(
        feeder: &mut Feeder,
        core: &mut ShellCore,
        mode: Option<WordMode>,
    ) -> Result<Option<Word>, ParseError> {
        if !Self::pre_check(feeder, &mode) {
            return Ok(None);
        }

        let mut ans = Word::default();
        if let Some(WordMode::Alias) = mode {
            let len = feeder.scanner_blank(core);
            ans.text = feeder.consume(len);
        }

        while let Some(sw) = subword::parse(feeder, core, &mode)? {
            match sw.is_extglob() {
                false => ans.push(sw),
                true => {
                    ans.text += sw.get_text();
                    ans.subwords.append(&mut sw.get_child_subwords());
                }
            }

            if !Self::post_check(feeder, core, &mode) {
                break;
            }
        }

        match ans.subwords.len() {
            0 => Ok(None),
            _ => Ok(Some(ans)),
        }
    }
}
