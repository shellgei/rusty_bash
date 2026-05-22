//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

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
    Exclude(Vec<String>),
}


impl WordMode {
    pub fn pre_check(&self, feeder: &mut Feeder) -> bool {
        if feeder.is_empty() {
            return false;
        }

        match self {
            Self::Arithmetic | Self::CompgenF => {
                if feeder.starts_with("}") {
                    return false;
                }
            }
            Self::Exclude(v) => {
                if feeder.starts_withs(v) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    pub fn post_check(&self, feeder: &mut Feeder, core: &mut ShellCore) -> bool {
        match self {
            WordMode::Arithmetic | WordMode::CompgenF => {
                if feeder.starts_withs(&["]", "}"]) || feeder.scanner_math_symbol(core) != 0 {
                    return false;
                }
            },
            WordMode::Exclude(v) => {
                if feeder.starts_withs(v) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }
}
