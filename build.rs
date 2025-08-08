//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

fn main() {
    // BASH_VERSION, BASH_VERSINFO[4]
    let profile = env::var("PROFILE").unwrap_or("".to_string());
    println!("cargo:rustc-env=CARGO_BUILD_PROFILE={profile}");
    // HOSTTYPE, MACHTYPE, BASH_VERSINFO[5]
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_ARCH={target_arch}");
    // MACHTYPE, BASH_VERSINFO[5]
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_VENDOR={target_vendor}");
    // OSTYPE, MACHTYPE, BASH_VERSINFO[5]
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_OS={target_os}");
}
