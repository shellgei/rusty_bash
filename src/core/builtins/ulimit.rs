//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, ShellCore};
use nix::libc;
use nix::sys::resource;
use nix::sys::resource::Resource;

const ITEMS: [(&str, &str, &str, Resource); 17] = [
        ("real-time non-blocking time  ","microseconds", "-R", Resource::RLIMIT_RTTIME),
        ("core file size              ","blocks", "-c", Resource::RLIMIT_CORE),
        ("data seg size               ","kbytes", "-d", Resource::RLIMIT_DATA),
        ("scheduling priority                 ","", "-e", Resource::RLIMIT_NICE),
        ("file size                   ","blocks", "-f", Resource::RLIMIT_FSIZE),
        ("pending signals                     ","", "-i", Resource::RLIMIT_SIGPENDING),
        ("max locked memory           ","kbytes", "-l", Resource::RLIMIT_MEMLOCK),
        ("max memory size             ","kbytes", "-m", Resource::RLIMIT_RSS),
        ("open files                          ","", "-n", Resource::RLIMIT_NOFILE),
        ("pipe size                ","512 bytes", "-p", Resource::RLIMIT_SIGPENDING), //dummy
        ("POSIX message queues         ","bytes", "-q", Resource::RLIMIT_MSGQUEUE),
        ("real-time priority                  ","", "-r", Resource::RLIMIT_RTPRIO),
        ("stack size                  ","kbytes", "-s", Resource::RLIMIT_STACK),
        ("cpu time                   ","seconds", "-t", Resource::RLIMIT_CPU),
        ("max user processes                  ","", "-u", Resource::RLIMIT_NPROC),
        ("virtual memory              ","kbytes", "-v", Resource::RLIMIT_AS),
        ("file locks                          ","", "-x", Resource::RLIMIT_LOCKS),
    ];

fn print_items(args: &Vec<String>, soft: bool) -> i32 {
    for a in args {
        for (item, unit, opt, key) in ITEMS {
            if a == opt {
                print_item(item, unit, opt, key, soft);
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
            v = (libc::PIPE_BUF as u64)/512;
        }

        if unit.starts_with("kbytes") {
            v /= 1024;
            infty /= 1024;
        }

        let s = if v == infty { "unlimited" } else { &v.to_string() };
    
        if unit == "" {
            println!("{}({}) {}", &item, &opt, &s);
        } else {
            println!("{}({}, {}) {}", &item, &unit, &opt, &s);
        }
    0
}

fn print_all(soft: bool) -> i32 {
    for (item, unit, opt, key) in ITEMS {
        print_item(item, unit, opt, key, soft);
    }
    0
}

fn set_limit(opt: &String, num: &String, soft: bool, hard: bool) -> i32 {
    let mut limit = match num.as_str() {
        "unlimited" => nix::sys::resource::RLIM_INFINITY,
        numstr => match numstr.parse::<u64>() {
            Ok(n) => n,
            Err(e) => {dbg!("{:?}", &e);return 1},
        }
    };

    for (_, unit, opt2, key) in ITEMS {
        if opt == opt2 {
            let (mut soft_limit, mut hard_limit) = resource::getrlimit(key).unwrap();

            if unit.starts_with("kbytes") {
                limit *= 1024;
            }

            if soft {
                soft_limit = limit;
            }
            if hard {
                hard_limit = limit;
            }

            match resource::setrlimit(key, soft_limit, hard_limit) {
                Err(e) => {dbg!("{:?}", &e);return 1},
                _ => return 0,
            }
        }
    }

    0
}

pub fn ulimit(_: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);
    let mut soft = arg::consume_option("-S", &mut args);
    let mut hard = arg::consume_option("-H", &mut args);

    if args.iter().any(|a| a == "-a"){
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

