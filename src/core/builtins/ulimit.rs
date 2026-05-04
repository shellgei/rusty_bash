//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, ShellCore};
use nix::libc;
use nix::sys::resource;
use nix::sys::resource::{Resource, rlim_t};

fn items() -> &'static [(&'static str, &'static str, &'static str, Resource)] {
    &[
        #[cfg(any(target_os = "linux"))]
        (
            "real-time non-blocking time  ",
            "microseconds",
            "-R",
            Resource::RLIMIT_RTTIME,
        ),
        (
            "core file size              ",
            "blocks",
            "-c",
            Resource::RLIMIT_CORE,
        ),
        (
            "data seg size               ",
            "kbytes",
            "-d",
            Resource::RLIMIT_DATA,
        ),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "scheduling priority                 ",
            "",
            "-e",
            Resource::RLIMIT_NICE,
        ),
        (
            "file size                   ",
            "blocks",
            "-f",
            Resource::RLIMIT_FSIZE,
        ),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "pending signals                     ",
            "",
            "-i",
            Resource::RLIMIT_SIGPENDING,
        ),
        (
            "max locked memory           ",
            "kbytes",
            "-l",
            Resource::RLIMIT_MEMLOCK,
        ),
        (
            "max memory size             ",
            "kbytes",
            "-m",
            Resource::RLIMIT_RSS,
        ),
        (
            "open files                          ",
            "",
            "-n",
            Resource::RLIMIT_NOFILE,
        ),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "pipe size                ",
            "512 bytes",
            "-p",
            Resource::RLIMIT_SIGPENDING,
        ), //dummy
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "POSIX message queues         ",
            "bytes",
            "-q",
            Resource::RLIMIT_MSGQUEUE,
        ),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "real-time priority                  ",
            "",
            "-r",
            Resource::RLIMIT_RTPRIO,
        ),
        (
            "stack size                  ",
            "kbytes",
            "-s",
            Resource::RLIMIT_STACK,
        ),
        (
            "cpu time                   ",
            "seconds",
            "-t",
            Resource::RLIMIT_CPU,
        ),
        (
            "max user processes                  ",
            "",
            "-u",
            Resource::RLIMIT_NPROC,
        ),
        #[cfg(not(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd")))]
        (
            "virtual memory              ",
            "kbytes",
            "-v",
            Resource::RLIMIT_AS,
        ),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        (
            "file locks                          ",
            "",
            "-x",
            Resource::RLIMIT_LOCKS,
        ),
    ]
}

fn print_items(args: &[String], soft: bool) -> i32 {
    for a in args {
        for (item, unit, opt, key) in items() {
            if a == opt {
                print_item(item, unit, opt, *key, soft);
            }
        }
    }

    0
}

fn print_item(item: &str, unit: &str, opt: &str, key: Resource, soft: bool) -> i32 {
    let (soft_limit, hard_limit) = resource::getrlimit(key).unwrap();
    let mut v = if soft { soft_limit } else { hard_limit };
    let mut infty = nix::sys::resource::RLIM_INFINITY;

    if item.starts_with("pipe size") {
        v = ((libc::PIPE_BUF as rlim_t) / 512) as rlim_t;
    }

    if unit.starts_with("kbytes") {
        v /= 1024;
        infty /= 1024;
    }

    let s = if v == infty {
        "unlimited"
    } else {
        &v.to_string()
    };

    if unit == "" {
        println!("{}({}) {}", &item, &opt, &s);
    } else {
        println!("{}({}, {}) {}", &item, &unit, &opt, &s);
    }
    0
}

fn print_all(soft: bool) -> i32 {
    for (item, unit, opt, key) in items() {
        print_item(item, unit, opt, *key, soft);
    }
    0
}

fn set_limit(opt: &String, num: &String, soft: bool, hard: bool) -> i32 {
    let mut limit = match num.as_str() {
        "unlimited" => nix::sys::resource::RLIM_INFINITY,
        numstr => match numstr.parse::<rlim_t>() {
            Ok(n) => n,
            Err(e) => {
                dbg!("{:?}", &e);
                return 1;
            }
        },
    };

    for (_, unit, opt2, key) in items() {
        if opt == opt2 {
            let (mut soft_limit, mut hard_limit) = resource::getrlimit(*key).unwrap();

            if unit.starts_with("kbytes") {
                limit *= 1024;
            }

            if soft {
                soft_limit = limit;
            }
            if hard {
                hard_limit = limit;
            }

            match resource::setrlimit(*key, soft_limit, hard_limit) {
                Err(e) => {
                    dbg!("{:?}", &e);
                    return 1;
                }
                _ => return 0,
            }
        }
    }

    0
}

pub fn ulimit(_: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(args);
    let mut soft = arg::consume_arg("-S", &mut args);
    let mut hard = arg::consume_arg("-H", &mut args);

    if args.iter().any(|a| a == "-a") {
        return print_all(!hard);
    }

    if args.len() > 2 && args[2].parse::<usize>().is_ok() {
        if !soft && !hard {
            soft = true;
            hard = true;
        }
        return set_limit(&args[1], &args[2], soft, hard);
    }

    print_items(&args, !hard)
}
