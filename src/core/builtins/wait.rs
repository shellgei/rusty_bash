//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn wait(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    for job in core.job_table.iter_mut() {
        job.update_status(true);
    }

    0
}
