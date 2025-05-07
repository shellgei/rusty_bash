//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::utils::exit;
use crate::elements::subword::CommandSubstitution;
use super::{BracedParam, EscapedChar, Parameter, Subword, VarName};

#[derive(Debug, Clone)]
pub struct ExtGlob {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for ExtGlob {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn get_child_subwords(&self) -> Vec<Box<dyn Subword>> { self.subwords.clone() }
    fn is_extglob(&self) -> bool {true}
    fn split(&self, _: &str, _: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ vec![] }
}

impl ExtGlob {
    pub fn new() -> ExtGlob {
        ExtGlob {
            text: String::new(),
            subwords: vec![],
        }
    }

    fn set_simple_subword(feeder: &mut Feeder, ans: &mut Self, len: usize) -> bool {
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push( From::from(&txt) );
        true
    }

    fn eat_braced_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        if let Some(a) = BracedParam::parse(feeder, core)? {
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_command_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        if let Some(a) = CommandSubstitution::parse(feeder, core)? {
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_special_or_positional_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = Parameter::parse(feeder, core){
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            return true;
        }

        false
    }

    fn eat_extglob(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        if let Some(a) = Self::parse(feeder, core)? {
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_doller(feeder: &mut Feeder, ans: &mut Self) -> bool {
        match feeder.starts_with("$") {
            true  => Self::set_simple_subword(feeder, ans, 1),
            false => false,
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self) -> bool {
        let len = feeder.scanner_subword_symbol();
        Self::set_simple_subword(feeder, ans, len)
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with("\\$") || feeder.starts_with("\\\\") {
            let txt = feeder.consume(2);
            ans.text += &txt;
            ans.subwords.push(Box::new(EscapedChar{ text: txt }));
            return true;
        }
        let len = feeder.scanner_escaped_char(core);
        Self::set_simple_subword(feeder, ans, len)
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push(Box::new( VarName{ text: txt}));
        true
    }

    fn eat_other(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_extglob_subword(core);
        Self::set_simple_subword(feeder, ans, len)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! core.shopts.query("extglob") 
        || feeder.scanner_extglob_head() == 0 {
            return Ok(None);
        }

        let mut ans = Self::new();
        ans.text = feeder.consume(2);
        ans.subwords.push( From::from(&ans.text) );

        loop {
            while Self::eat_braced_param(feeder, &mut ans, core)?
               || Self::eat_command_substitution(feeder, &mut ans, core)?
               || Self::eat_extglob(feeder, &mut ans, core)?
               || Self::eat_special_or_positional_param(feeder, &mut ans, core)
               || Self::eat_doller(feeder, &mut ans)
               || Self::eat_escaped_char(feeder, &mut ans, core)
               || Self::eat_name(feeder, &mut ans, core)
               || Self::eat_symbol(feeder, &mut ans)
               || Self::eat_other(feeder, &mut ans, core) {}

            if feeder.starts_with(")") {
                ans.text += &feeder.consume(1);
                ans.subwords.push( From::from(")") );
                return Ok(Some(ans));
            }else if feeder.starts_with("|") {
                ans.text += &feeder.consume(1);
                ans.subwords.push( From::from("|") );
            }else if feeder.len() > 0 {
                exit::internal("unknown chars in double quoted word");
            }else if ! feeder.feed_additional_line(core).is_ok() {
                return Ok(None);
            }
        }
    }
}
