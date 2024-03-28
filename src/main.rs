//SPDX-FileCopyrightText: 2024 Ryuichi Ueda
//SPDX-License-Identifier: BSD-3-Clause

mod core;
mod feeder;

use crate::core::ShellCore;
use crate::feeder::{InputError, Feeder};
use std::{env, process};

fn show_version() {
    const S: &str = "Sushi Shell book version
© 2024 Ryuichi Ueda
License: BSD 3-Clause

This is open source software. You can redistirbute and use in source
and binary forms with or without modification under the license.
There is no warranty, to the extent permitted by law.";
    println!("{}", S);
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    let mut core = ShellCore::new();
    main_loop(&mut core);
}

fn main_loop(core: &mut ShellCore) {
    let mut feeder = Feeder::new();
    loop {
        match feeder.feed_line(core) {
            Ok(()) => {},
            Err(InputError::Eof) => break,
        }
    }
}
