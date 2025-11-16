//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use cargo_toml::Manifest;
use std::{env, path::PathBuf};

fn main() {
    // SUSH_VERSION, SUSH_VERSINFO[4]
    let profile = env::var("PROFILE").unwrap_or("".to_string());
    println!("cargo:rustc-env=CARGO_BUILD_PROFILE={profile}");
    // HOSTTYPE, MACHTYPE, BASH_VERSINFO[5], SUSH_VERSINFO[5]
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_ARCH={target_arch}");
    // MACHTYPE, BASH_VERSINFO[5], SUSH_VERSINFO[5]
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_VENDOR={target_vendor}");
    // OSTYPE, MACHTYPE, BASH_VERSINFO[5], SUSH_VERSINFO[5]
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=CARGO_CFG_TARGET_OS={target_os}");

    // metadata
    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let manifest = Manifest::from_path(&manifest_path).expect("failed to parse Cargo.toml");
    // compat
    let compat = manifest
        .package
        .as_ref()
        .and_then(|p| p.metadata.as_ref()?.get("compat"))
        .and_then(|v| v.as_table())
        .expect("Missing [package.metadata.compat]");
    for (k, v) in compat {
        let env_key: String = k
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .to_ascii_uppercase();
        let val = v
            .as_str()
            .unwrap_or_else(|| panic!("Non-string value for key {k} in [package.metadata.compat]"));
        println!("cargo:rustc-env=COMPAT_{env_key}={val}");
    }
}
