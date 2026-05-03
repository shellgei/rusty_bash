//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2026 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::arg;

pub fn history_c(core: &mut ShellCore) -> i32 {
    core.rewritten_history.clear();
    core.history.clear();
    0
}

pub fn history(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let mut args = arg::dissolve_options(&args);
    if arg::consume_arg("-c", &mut args) {
        return history_c(core);
    }

    if args.len() > 1 {
        let msg = format!("{}: invalid option", &args[1]);
        return super::error_(1, "history", &msg, core);
    }

    let mut number = 1;

    let filename = core.db.get_param("HISTFILE").unwrap_or_default();
    if filename.is_empty() {
        return 0;
    }

    if let Ok(history) = sushline::readline::History::read_file(&filename) {
        for entry in history.entries() {
            println!("{:5} {}", number, entry.line());
            number += 1;
        }
    }

    for h in core.history.iter().rev() {
        println!("{:5} {}", number, h);
        number += 1;
    }

    0
}
