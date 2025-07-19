//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, ShellCore};
use nix::sys::resource;
use nix::sys::resource::Resource;

fn print(soft: bool) -> i32 {
    let items = [
        (Resource::RLIMIT_RTTIME, "real-time non-blocking time  (microseconds, -R)")
    ];

    for (key, item) in items {
        let (soft_limit, hard_limit) = resource::getrlimit(key).unwrap();

        let v = if soft { soft_limit } else { hard_limit };
        let s = match v {
            nix::sys::resource::RLIM_INFINITY => "unlimited",
            _ => &soft_limit.to_string(),
        };
    
        println!("{} {}", &item, &s);
    }
    0
}

pub fn ulimit(_: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let _ = arg::dissolve_options(args);

    if args.iter().any(|a| a == "-a"){
        return print(true);
    }

    0
}

