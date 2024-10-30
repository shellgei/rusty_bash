use std::env;

fn main() {
    // BASH_VERSION, BASH_VERSINFO[4]
    let profile = env::var("PROFILE").unwrap_or("".to_string());
    println!("cargo:rustc-env=CARGO_BUILD_PROFILE={}", profile);
    // MACHTYPE, BASH_VERSINFO[5]
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or("unknown".to_string());
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or("unknown".to_string());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or("unknown".to_string());    
    let target_gnu_format = format!("{}-{}-{}", target_arch, target_vendor, target_os);
    println!("cargo:rustc-env=CARGO_CFG_TARGET_GNU_FORMAT={}", target_gnu_format);
}