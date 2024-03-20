//SPDX-FileCopyrightText: 2024 Ryuichi Ueda
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
}
