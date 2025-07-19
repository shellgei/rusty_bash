//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{arg, ShellCore};
use nix::libc;
use nix::sys::resource;
use nix::sys::resource::Resource;

fn print(soft: bool) -> i32 {
    let items = [
        ("real-time non-blocking time  ","microseconds, -R", Resource::RLIMIT_RTTIME),
        ("core file size              ","blocks, -c", Resource::RLIMIT_CORE),
        ("data seg size               ","kbytes, -d", Resource::RLIMIT_DATA),
        ("scheduling priority                 ","-e", Resource::RLIMIT_NICE),
        ("file size                   ","blocks, -f", Resource::RLIMIT_FSIZE),
        ("pending signals                     ","-i", Resource::RLIMIT_SIGPENDING),
        ("max locked memory           ","kbytes, -l", Resource::RLIMIT_MEMLOCK),
        ("max memory size             ","kbytes, -m", Resource::RLIMIT_RSS),
        ("open files                          ","-n", Resource::RLIMIT_NOFILE),
        ("pipe size                ","512 bytes, -p", Resource::RLIMIT_SIGPENDING), //dummy
        ("POSIX message queues         ","bytes, -q", Resource::RLIMIT_MSGQUEUE),
        ("real-time priority                  ","-r", Resource::RLIMIT_RTPRIO),
        ("stack size                  ","kbytes, -s", Resource::RLIMIT_STACK),
        ("cpu time                   ","seconds, -t", Resource::RLIMIT_CPU),
        ("max user processes                  ","-u", Resource::RLIMIT_NPROC),
        ("virtual memory              ","kbytes, -v", Resource::RLIMIT_AS),
        ("file locks                          ","-x", Resource::RLIMIT_LOCKS),
    ];

    for (item, unit, key) in items {
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
    
        println!("{}({}) {}", &item, &unit, &s);
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

