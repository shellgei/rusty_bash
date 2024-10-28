use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-env=CARGO_BUILD_PROFILE={}", profile);
    }
}