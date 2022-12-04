//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause
//
pub mod script;

pub mod function;
pub mod pipeline;

pub mod command;
pub mod compound_brace;
pub mod compound_case;
pub mod compound_double_paren;
pub mod compound_if;
pub mod compound_paren;
pub mod compound_while;

pub mod substitution;

pub mod word;
pub mod value;

pub mod subword_braced;
pub mod subword_command_substitution;
pub mod subword_double_quoted;
pub mod subword_math_substitution;
pub mod subword_string_double_quoted;
pub mod subword_string_non_quoted;
pub mod subword_single_quoted;
pub mod subword_tilde;
pub mod subword_variable;

pub mod redirect;
