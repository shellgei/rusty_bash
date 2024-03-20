//SPDX-FileCopyrightText: 2024 Ryuichi Ueda
//SPDX-License-Identifier: BSD-3-Clause

use std::{env, process};

fn show_version() {
    eprintln!("Sushi Shell book version");
    eprintln!("© 2024 Ryuichi Ueda");
    eprintln!("License: BSD 3-Clause\n");

    eprintln!("This is open source software. You can redistirbute and use in source\nand binary forms with or without modification under the license.");
    eprintln!("There is no warranty, to the extent permitted by law.");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        show_version();
    }

    println!("Hello, world!");
}
