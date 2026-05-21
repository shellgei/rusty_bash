//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

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
