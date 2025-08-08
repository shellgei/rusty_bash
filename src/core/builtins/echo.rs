//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::ansi_c_str::AnsiCString;
use crate::error::exec::ExecError;
use crate::utils::c_string;
use crate::{Feeder, ShellCore};
use std::ffi::CString;
use std::io;
use std::io::{stdout, Write};

fn arg_to_c_str(arg: &String, core: &mut ShellCore) -> Result<CString, ExecError> {
    let mut f = Feeder::new(arg);
    let ans = match AnsiCString::parse(&mut f, core, true) {
        Ok(Some(mut ansi_c_str)) => c_string::to_carg(&ansi_c_str.eval()),
        Ok(None) => c_string::to_carg(arg),
        Err(e) => return Err(ExecError::ParseError(e)),
    };

    Ok(ans)
}

pub fn echo(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut first = true;
    let mut e_opt = false;
    let mut n_opt = false;

    if args.len() == 1 {
        println!();
        return 0;
    }

    match args[1].as_ref() {
        "-ne" | "-en" => {
            e_opt = true;
            n_opt = true;
            args.remove(1);
        }
        "-e" => {
            e_opt = true;
            args.remove(1);
        }
        "-n" => {
            n_opt = true;
            args.remove(1);
        }
        _ => {}
    }

    for a in &args[1..] {
        if !first {
            let _ = io::stdout().write(b" ");
        }
        first = false;

        let bytes = match e_opt {
            false => c_string::to_carg(a).into_bytes(),
            true => match arg_to_c_str(a, core) {
                Ok(v) => v.into_bytes(),
                Err(e) => {
                    e.print(core);
                    return 1;
                }
            },
        };

        io::stdout().write_all(&bytes).unwrap();
    }

    if !n_opt {
        let _ = io::stdout().write(b"\n");
    }

    stdout().flush().unwrap();
    0
}
