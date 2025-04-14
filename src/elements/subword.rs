//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod ansi_c_quoted;
pub mod simple;
pub mod single_quoted;
mod braced_param;
mod command_sub;
mod escaped_char;
mod ext_glob;
mod double_quoted;
pub mod parameter;
mod varname;
mod arithmetic;
pub mod filler;
mod process_sub;

use crate::{ShellCore, Feeder};
use crate::error::{exec::ExecError, parse::ParseError};
use crate::elements::word::WordMode;
use crate::utils::splitter;
use self::ansi_c_quoted::AnsiCQuoted;
use self::arithmetic::Arithmetic;
use self::simple::SimpleSubword;
use self::braced_param::BracedParam;
use self::command_sub::CommandSubstitution;
use self::escaped_char::EscapedChar;
use self::ext_glob::ExtGlob;
use self::filler::FillerSubword;
use self::double_quoted::DoubleQuoted;
use self::single_quoted::SingleQuoted;
use self::process_sub::ProcessSubstitution;
use self::parameter::Parameter;
use self::varname::VarName;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

impl Clone for Box::<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}


fn split(sw: &str, ifs: &str, prev_char: Option<char>) -> Vec<(String, bool)>{ //bool: true if it should remain
    if ifs == "" {
        return vec![(sw.to_string(), false)];
    }

    if ifs.chars().all(|c| " \t\n".contains(c) ) {
        splitter::split_str_normal(sw, ifs)
    }else {
        splitter::split_str_special(sw, ifs, prev_char)
    }
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn set_text(&mut self, _: &str) {}
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn substitute(&mut self, _: &mut ShellCore) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        Ok(vec![]) // return subwords if the self object must be replaced to them
    }

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)>{ //bool: true if it should remain
        let f = |s| Box::new( SimpleSubword {text: s}) as Box<dyn Subword>;

        let ans = split(&self.get_text(), ifs, prev_char);
        ans.iter().map(|s| (f(s.0.to_string()), s.1)).collect()
    }

    fn make_glob_string(&mut self) -> String {self.get_text().to_string()}

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s  => Some(s.to_string()),
        }
    }

    fn make_regex(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s  => Some(s.to_string()),
        }
    }

    fn is_name(&self) -> bool {false}
    fn is_array(&self) -> bool {false}
    fn get_array_elem(&self) -> Vec<String> {vec![]}
    fn is_extglob(&self) -> bool {false}
    fn get_child_subwords(&self) -> Vec<Box<dyn Subword>> { vec![] }
}

fn replace_history_expansion(feeder: &mut Feeder, core: &mut ShellCore) -> bool {
    let len = feeder.scanner_history_expansion(core);
    if len == 0 {
        return false;
    }

    let history_len = core.history.len();
    if history_len < 2 {
        feeder.replace(len, "");
        return true;
    }

    let mut his = String::new();
    for h in &core.history[1..] {
        let last = h.split(" ").last().unwrap();

        if ! last.starts_with("!$") {
            his = last.to_string();
            break;
        }
    }

    feeder.replace(len, &his);
    true
}

pub fn parse_special_subword(feeder: &mut Feeder,core: &mut ShellCore,
    mode: &Option<WordMode>) -> Result<Option<Box<dyn Subword>>, ParseError> {
    match mode {
        None => Ok(None),
        Some(WordMode::ParamOption(ref v)) => {
            if feeder.len() == 0 || feeder.starts_withs2(v) {
                return Ok(None);
            }

            let len = feeder.scanner_char();
            let c = FillerSubword { text: feeder.consume(len) };
            if feeder.len() == 0 {
                feeder.feed_additional_line(core)?;
            }
            Ok(Some(Box::new(c)))
        },
        Some(WordMode::ReadCommand) => {
            if feeder.len() == 0 
            || feeder.starts_withs(&["\n", "\t", " "]) {
                Ok(None)
            }else{
                let c = SimpleSubword { text: feeder.consume(1) };
                Ok(Some(Box::new(c)))
            }
        },
        _ => Ok(None),
    }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, mode: &Option<WordMode>)
                                    -> Result<Option<Box<dyn Subword>>, ParseError> {
    if replace_history_expansion(feeder, core) {
        return parse(feeder, core, mode);
    }

    if let Some(a) = BracedParam::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = AnsiCQuoted::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = Arithmetic::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = CommandSubstitution::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = ProcessSubstitution::parse(feeder, core, mode)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = SingleQuoted::parse(feeder, core){ Ok(Some(Box::new(a))) }
    else if let Some(a) = DoubleQuoted::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = ExtGlob::parse(feeder, core)? { Ok(Some(Box::new(a))) }
    else if let Some(a) = EscapedChar::parse(feeder, core){ Ok(Some(Box::new(a))) }
    else if let Some(a) = Parameter::parse(feeder, core){ Ok(Some(Box::new(a))) }
    else if let Some(a) = VarName::parse(feeder, core){ Ok(Some(Box::new(a))) }
    else if let Some(a) = SimpleSubword::parse(feeder){ Ok(Some(Box::new(a))) }
    else{ parse_special_subword(feeder, core, mode) }
}
