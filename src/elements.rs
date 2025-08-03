//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod command;
pub mod expr;
pub mod io;
pub mod job;
pub mod pipeline;
pub mod script;
pub mod substitution;
pub mod subword;
pub mod word;

use self::io::pipe::Pipe;
