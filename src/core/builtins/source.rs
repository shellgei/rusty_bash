//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{file_check, Script, ShellCore, Feeder};
use crate::error::input::InputError;
use crate::error::parse::ParseError;
use crate::elements::io;
use std::fs::File;
use std::os::fd::IntoRawFd;

pub fn source(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        eprintln!("sush: source: filename argument required");
        eprintln!("source: usage: source filename [arguments]");
        return 2;
    }

    if file_check::is_dir(&args[1]) {
        eprintln!("sush: source: {}: is a directory", &args[1]);
        return 1;
    }

    let file = match File::open(&args[1]) {
        Ok(f)  => f, 
        Err(e) => {
            eprintln!("sush: {}: {}", &args[1], &e);
            return 1;
        }, 
    };

    let fd = file.into_raw_fd();
    let backup = io::backup(0);
    io::replace(fd, 0);
    let read_stdin_backup = core.read_stdin;
    core.read_stdin = true;
    core.source_function_level += 1;
    core.source_level += 1;

    let mut feeder = Feeder::new("");
    loop {
        match feeder.feed_line(core) {
            Ok(()) => {}, 
            _ => break,
        }

        if core.return_flag {
            feeder.consume(feeder.len());
        }

        match Script::parse(&mut feeder, core, false){
            Ok(Some(mut s)) => s.exec(core),
            _ => {},
        }
    }

    io::replace(backup, 0);
    core.source_function_level -= 1;
    core.source_level -= 1;
    core.return_flag = false;
    core.read_stdin = read_stdin_backup;
    core.db.exit_status
}
