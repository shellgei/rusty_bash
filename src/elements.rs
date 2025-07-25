//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod ansi_c_str;
pub mod script;
pub mod job;
pub mod pipeline;
pub mod command;
pub mod io;
pub mod word;
pub mod substitution;
pub mod subword;
pub mod expr;

use self::io::pipe::Pipe;
