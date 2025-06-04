//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore, Script};

pub fn eval(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    args.remove(0);
    dbg!("{:?}", &args);
    if args.len() > 0 && args[0] == "--" {
        args.remove(0);
    }

    let mut feeder = Feeder::new(&args.join(" "));

    core.eval_level += 1;
    match Script::parse(&mut feeder, core, false){
        Ok(Some(mut s)) => {
            let _ = s.exec(core);
        },
        Err(e) => e.print(core),
        _        => {},
    }

    core.eval_level -= 1;
    core.db.exit_status
}
