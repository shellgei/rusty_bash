//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: Word,
    pub evaluated_string: String,
}
