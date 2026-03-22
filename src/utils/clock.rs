//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org, @ru@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use nix::time;
use nix::sys::resource;
use nix::sys::resource::UsageWho;
use nix::sys::time::TimeVal;
use nix::time::ClockId;
use std::time::Duration;

pub fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(
        now.tv_sec().try_into().unwrap(),
        now.tv_nsec().try_into().unwrap(),
    )
}

pub fn get_epochseconds() -> String {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    real.tv_sec().to_string()
}

pub fn get_epochrealtime() -> String {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    format!("{}.{:06}", real.tv_sec(), real.tv_nsec() / 1000).to_string()
}

pub fn get_user_and_sys() -> (TimeVal, TimeVal) {
    let sush = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
    let children = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

    let user = sush.user_time() + children.user_time();
    let sys = sush.system_time() + children.system_time();

    (user, sys)
}
