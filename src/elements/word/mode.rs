//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::word::Subword;
use crate::elements::subword::filler::FillerSubword;
use crate::error::parse::ParseError;

#[derive(Debug, Clone)]
pub enum WordMode {
    Alias,
    Arithmetic,
    AssocIndex,
    EvalLet,
    CompgenF,
    ReadCommand,
    Heredoc,
    RightOfSubstitution,
    Value,
    PermitAnyChar,
    Exclude(Vec<String>),
}


impl WordMode {
    pub fn pre_check(&self, feeder: &mut Feeder) -> bool {
        if feeder.is_empty() {
            return false;
        }

        match self {
            Self::Arithmetic | Self::CompgenF => ! feeder.starts_with("}"),
            Self::Exclude(v) => ! feeder.starts_withs(v),
            _ => true,
        }
    }

    pub fn post_check(&self, feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        match self {
            WordMode::Arithmetic | WordMode::CompgenF => 
                ! feeder.starts_withs(&["]", "}"]) && feeder.scanner_math_symbol(core) == 0,
            WordMode::Exclude(v) => ! feeder.starts_withs(v),
            _ => true,
        }
    }

    pub fn last_resort(&self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Box<dyn Subword>>, ParseError> {
        match self {
            WordMode::Exclude(v) => {
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
            WordMode::ReadCommand => {
                if feeder.is_empty() || feeder.starts_withs(&["\n", "\t", " "]) {
                    Ok(None)
                } else {
                    Ok(Some(From::from(&feeder.consume(1))))
                }
            }
            WordMode::Alias => {
                if feeder.starts_with("\t") {
                    Ok(Some(From::from(&feeder.consume(1))))
                } else {
                    Ok(None)
                }
            }
            WordMode::AssocIndex => {
                if !feeder.starts_with("]") {
                    Ok(Some(From::from(&feeder.consume(1))))
                } else {
                    Ok(None)
                }
            }
            WordMode::PermitAnyChar => {
                if feeder.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(From::from(&feeder.consume(1))))
                }
            }
            _ => Ok(None),
        }
    }
}
