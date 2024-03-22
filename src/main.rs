//SPDX-FileCopyrightText: 2024 Ryuichi Ueda
//SPDX-License-Identifier: BSD-3-Clause

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

    println!("--versionを指定すると、ここには来ません！");
}
