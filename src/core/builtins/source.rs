//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::error::parse::ParseError;
use crate::{file_check, Feeder, Script, ShellCore};

fn check_error(core: &mut ShellCore, args: &[String]) -> i32 {
    if core.db.flags.contains('r') && args[1].contains('/') {
        let msg = format!("{}: restricted", &args[1]);
        return super::error_exit(1, &args[0], &msg, core);
    }

    if args.len() < 2 {
        eprintln!("sush: source: filename argument required");
        eprintln!("source: usage: source filename [arguments]");
        return 2;
    }

    if file_check::is_dir(&args[1]) {
        eprintln!("sush: source: {}: is a directory", &args[1]);
        return 1;
    }
    0
}

pub fn source(core: &mut ShellCore, args: &[String]) -> i32 {
    let args = args.to_owned();
    let check = check_error(core, &args);
    if check != 0 {
        return check;
    }

    let mut feeder = Feeder::new("");
    if let Err(e) = feeder.set_file(&args[1]) {
        ParseError::Input(e).print(core);
        return 1;
    }

    let mut source = match core.db.get_vec("BASH_SOURCE", false) {
        Ok(s) => s,
        Err(e) => {
            e.print(core);
            return 1;
        }
    };

    core.source_function_level += 1;
    core.source_files.push(args[1].to_string());
    core.db.position_parameters.push(args[1..].to_vec());
    source.insert(0, args[1].clone());
    let _ = core.db.set_array("BASH_SOURCE", Some(source.clone()), None);

    feeder.main_feeder = true;
    while let Ok(()) = feeder.feed_line(core) {
        if core.return_flag {
            feeder.consume(feeder.len());
        }

        match Script::parse(&mut feeder, core, false) {
            Ok(Some(mut s)) => {
                let _ = s.exec(core);
            }
            Err(e) => e.print(core),
            _ => {}
        }
    }

    source.remove(0);
    let _ = core.db.set_array("BASH_SOURCE", Some(source), None);
    core.db.position_parameters.pop();
    core.source_function_level -= 1;
    core.source_files.pop();
    core.return_flag = false;
    core.db.exit_status
}
