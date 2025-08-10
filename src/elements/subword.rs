//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod ansi_c_quoted;
mod braced_param;
mod command_sub;
pub mod simple;
pub mod single_quoted;
//mod command_sub_old;
mod arithmetic;
mod double_quoted;
pub mod escaped_char;
mod ext_glob;
mod file_input;
pub mod filler;
pub mod parameter;
mod paren;
mod process_sub;
mod varname;

use self::ansi_c_quoted::AnsiCQuoted;
use self::arithmetic::Arithmetic;
use self::braced_param::BracedParam;
use self::command_sub::CommandSubstitution;
use self::simple::SimpleSubword;
use crate::elements::word::WordMode;
use crate::error::{exec::ExecError, parse::ParseError};
use crate::utils::splitter;
use crate::{Feeder, ShellCore};
//use self::command_sub_old::CommandSubstitutionOld;
use self::double_quoted::DoubleQuoted;
use self::escaped_char::EscapedChar;
use self::ext_glob::ExtGlob;
use self::file_input::FileInput;
use self::filler::FillerSubword;
use self::parameter::Parameter;
use self::paren::EvalLetParen;
use self::process_sub::ProcessSubstitution;
use self::single_quoted::SingleQuoted;
use self::varname::VarName;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(self.get_text()).finish()
    }
}

impl Clone for Box<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}

impl Default for Box<dyn Subword> {
    fn default() -> Box<dyn Subword> {
        Box::new(SimpleSubword {
            text: "".to_string(),
        })
    }
}

impl From<&String> for Box<dyn Subword> {
    fn from(s: &String) -> Box<dyn Subword> {
        Box::new(SimpleSubword { text: s.clone() })
    }
}

impl From<&str> for Box<dyn Subword> {
    fn from(s: &str) -> Box<dyn Subword> {
        Box::new(SimpleSubword {
            text: s.to_string(),
        })
    }
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn set_text(&mut self, _: &str) {}
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn substitute(&mut self, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }
    fn alter(&mut self) -> Result<Vec<Box<dyn Subword>>, ExecError> {
        Ok(vec![])
    }

    fn split(&self, ifs: &str, prev_char: Option<char>) -> Vec<(Box<dyn Subword>, bool)> {
        //bool: true if it should remain
        splitter::split(self.get_text(), ifs, prev_char)
            .iter()
            .map(|s| (From::from(&s.0), s.1))
            .collect()
    }

    fn make_glob_string(&mut self) -> String {
        self.get_text().to_string()
    }

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s => Some(s.to_string()),
        }
    }

    fn make_regex(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s => Some(s.to_string()),
        }
    }

    fn is_name(&self) -> bool {
        false
    }
    fn is_array(&self) -> bool {
        false
    }
    fn get_elem(&mut self) -> Vec<String> {
        vec![]
    }
    fn is_extglob(&self) -> bool {
        false
    }
    fn get_child_subwords(&self) -> Vec<Box<dyn Subword>> {
        vec![]
    }
    fn set_heredoc_flag(&mut self) {}
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

        if !last.starts_with("!$") {
            his = last.to_string();
            break;
        }
    }

    feeder.replace(len, &his);
    true
}

pub fn parse_special_subword(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    mode: &Option<WordMode>,
) -> Result<Option<Box<dyn Subword>>, ParseError> {
    match mode {
        None => Ok(None),
        Some(WordMode::ParamOption(ref v)) => {
            if feeder.is_empty() || feeder.starts_withs(v) {
                return Ok(None);
            }

            let len = feeder.scanner_char();
            let c = FillerSubword {
                text: feeder.consume(len),
            };
            if feeder.is_empty() {
                feeder.feed_additional_line(core)?;
            }
            Ok(Some(Box::new(c)))
        }
        Some(WordMode::ReadCommand) => {
            if feeder.is_empty() || feeder.starts_withs(&["\n", "\t", " "]) {
                Ok(None)
            } else {
                Ok(Some(From::from(&feeder.consume(1))))
            }
        }
        Some(WordMode::Alias) => {
            if feeder.starts_with("\t") {
                Ok(Some(From::from(&feeder.consume(1))))
            } else {
                Ok(None)
            }
        }
        Some(WordMode::AssocIndex) => {
            if !feeder.starts_with("]") {
                Ok(Some(From::from(&feeder.consume(1))))
            } else {
                Ok(None)
            }
        }
        Some(WordMode::ReparseOfValue) => {
            if feeder.is_empty() {
                Ok(None)
            } else {
                Ok(Some(From::from(&feeder.consume(1))))
            }
        }
        _ => Ok(None),
    }
}

pub fn parse(
    feeder: &mut Feeder,
    core: &mut ShellCore,
    mode: &Option<WordMode>,
) -> Result<Option<Box<dyn Subword>>, ParseError> {
    if replace_history_expansion(feeder, core) {
        return parse(feeder, core, mode);
    }

    if let Some(a) = BracedParam::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = AnsiCQuoted::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Arithmetic::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = FileInput::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = CommandSubstitution::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    }
    //else if let Some(a) = CommandSubstitutionOld::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = ProcessSubstitution::parse(feeder, core, mode)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = SingleQuoted::parse(feeder, core) {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = DoubleQuoted::parse(feeder, core, mode)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = ExtGlob::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = EscapedChar::parse(feeder, core) {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Parameter::parse(feeder, core) {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = VarName::parse(feeder, core) {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = SimpleSubword::parse(feeder) {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = EvalLetParen::parse(feeder, core, mode)? {
        Ok(Some(Box::new(a)))
    } else {
        parse_special_subword(feeder, core, mode)
    }
}
