//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::substitution::Substitution;
use crate::error::exec::ExecError;

fn declare_print_params(core: &mut ShellCore) -> i32 {
    for key in core.db.get_param_keys() {
        let value = core.db.get_param(&key).unwrap();
        println!("{key}={value}");
    }

    0
}

fn declare_print(core: &mut ShellCore) -> i32 {
    for key in core.db.get_param_keys() {
        let value = core.db.get_param(&key).unwrap();
        println!("{key}={value}");
    }

    0
}

pub fn declare(core: &mut ShellCore, args: &[String],
               subs: &mut [Substitution]) -> i32 {
    if args.len() == 1 && subs.is_empty() {
        return declare_print(core);
    }
    0
}

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

pub fn readonly(core: &mut ShellCore, args: &[String],
                subs: &mut [Substitution]) -> i32 {
    for sub in subs.iter_mut() {
        let layer = core.db.solve_set_layer(&sub.left_hand.text, None);
        if let Err(e) = sub.eval(core, Some(layer)) {
            return super::error_exit(1, &args[0], &e, core);
        }

        core.db.set_flag(&sub.left_hand.text, 'r', layer);
    }

    0
}
