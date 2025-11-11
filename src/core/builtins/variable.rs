//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;

pub fn local(core: &mut ShellCore, args: &[String],
             subs: &mut [Substitution]) -> i32 {
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2
    } else {
        let e = &ExecError::ValidOnlyInFunction;
        return super::error_exit(1, &args[0], e, core);
    };

    for sub in subs.iter_mut() {
        if let Err(e) = sub.eval(core, Some(layer)) {
            return super::error_exit(1, &args[0], &e, core);
        }
    }

    0
}
