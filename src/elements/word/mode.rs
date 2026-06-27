//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::filler::FillerSubword;
use crate::elements::word::Subword;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

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
    PermitAnyUntil(Vec<String>),
}

impl WordMode {
    pub fn word_pre_check(&self, feeder: &mut Feeder) -> bool {
        if feeder.is_empty() {
            return false;
        }

        match self {
            Self::Arithmetic | Self::CompgenF => !feeder.starts_with("}"),
            Self::PermitAnyUntil(v) => !feeder.starts_with_one_of(v),
            _ => true,
        }
    }

    pub fn word_post_check(&self, feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        match self {
            WordMode::Arithmetic | WordMode::CompgenF => {
                !feeder.starts_with_one_of(&["]", "}"]) && feeder.scanner_math_symbol(core) == 0
            }
            WordMode::PermitAnyUntil(v) => !feeder.starts_with_one_of(v),
            _ => true,
        }
    }

    pub fn subword_post_check(
        &self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
    ) -> Result<Option<Box<dyn Subword>>, ParseError> {
        match self {
            WordMode::PermitAnyUntil(v) => {
                if feeder.is_empty() || feeder.starts_with_one_of(v) {
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
                match feeder.is_empty() || feeder.starts_with_one_of(&["\n", "\t", " "]) {
                    true => Ok(None),
                    false => Ok(Some(From::from(&feeder.consume(1)))),
                }
            }
            WordMode::Alias => match feeder.starts_with("\t") {
                true => Ok(Some(From::from(&feeder.consume(1)))),
                false => Ok(None),
            },
            WordMode::AssocIndex => match feeder.starts_with("]") {
                false => Ok(Some(From::from(&feeder.consume(1)))),
                true => Ok(None),
            },
            WordMode::PermitAnyChar => match feeder.is_empty() {
                true => Ok(None),
                false => Ok(Some(From::from(&feeder.consume(1)))),
            },
            _ => Ok(None),
        }
    }
}
