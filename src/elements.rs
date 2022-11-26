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

pub mod arg;
pub mod value;

pub mod subarg_braced;
pub mod subarg_command_substitution;
pub mod subarg_double_quoted;
pub mod subarg_math_substitution;
pub mod subarg_string_double_quoted;
pub mod subarg_string_non_quoted;
pub mod subarg_single_quoted;
pub mod subarg_tilde;
pub mod subarg_variable;

pub mod redirect;

pub mod varname;

