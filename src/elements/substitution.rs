//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod value;
mod variable;

use self::value::Value;
use self::variable::Variable;

#[derive(Debug, Clone, Default)]
pub struct Substitution {
    pub text: String,
    left_hand: Variable,
    right_hand: Value,
}
