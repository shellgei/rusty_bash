//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use super::{Arc, AtomicBool};

#[derive(Default)]
pub struct Trap {
    pub list: Vec<(i32, String)>,
    pub trapped: Vec<(Arc<AtomicBool>, String)>,
    pub exit_script: String,
    pub exit_script_run: bool,
    pub debug_script: String,
    pub debug_script_run: bool,
    pub error_script: String,
    pub error_script_run: bool,
}
