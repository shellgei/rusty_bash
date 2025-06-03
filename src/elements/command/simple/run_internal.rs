//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::SimpleCommand;
use crate::error::exec::ExecError;

pub fn run(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<bool, ExecError> {
    let ans = run_function(&mut com.args, core) 
           || core.run_substitution_builtin(&mut com.args,
                &mut com.substitutions_as_args)?
           || core.run_builtin(&mut com.args,
                &com.substitutions_as_args)?;
    Ok(ans)
}

fn run_function(args: &mut Vec<String>, core: &mut ShellCore) -> bool {
    match core.db.functions.get_mut(&args[0]) {
        Some(f) => {f.clone().run_as_command(args, core); true},
        None => false,
    }
}
