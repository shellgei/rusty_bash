//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod arg;
pub mod c_string;
pub mod clock;
pub mod directory;
pub mod exit;
pub mod file_check;
pub mod glob;

pub fn reserved(w: &str) -> bool {
    matches!(w, "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi")
}
