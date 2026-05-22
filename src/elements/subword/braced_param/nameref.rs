//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils};
use super::{ExecError, Variable};

pub fn solve(var: &mut Variable, core: &mut ShellCore) -> Result<(), ExecError> {
    let mut circular_check_vec = vec![];
    let org_name = var.name.clone();
    loop {
        let bkup = var.name.clone();
        var.check_nameref(core)?;
        if var.name == bkup {
            var.name = utils::gen_not_exist_var(core);
        }

        if circular_check_vec.contains(&var.name) {
            ExecError::CircularNameRef(org_name).print(core);
            var.name = utils::gen_not_exist_var(core);
            break;
        }
        if ! core.db.exist_nameref(&var.name) {
            break;
        }
        circular_check_vec.push(var.name.clone());
    }

    Ok(())
}
