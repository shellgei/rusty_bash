//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::time::Duration;
use nix::time;
use nix::time::ClockId;

pub fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(now.tv_sec().try_into().unwrap(), now.tv_nsec().try_into().unwrap())
}

/*
pub fn set_seconds() -> String {
    let offset = Duration::seconds(0);
    let adjusted = monotonic_time() - offset;
    format!("{}.{}", adjusted.whole_seconds(), adjusted.subsec_nanoseconds())
}

pub fn get_seconds(v: &mut Vec<String>) -> String {
    if v.is_empty() {
        v.push(set_seconds());
    }

    let part: Vec<&str> = v[0].split('.').collect();
    let sec = i64::from_str(part[0]).unwrap();
    let nano = i32::from_str(part[1]).unwrap();
    let offset = Duration::new(sec, nano);
    let elapsed = monotonic_time() - offset;
    let ans = elapsed.whole_seconds().to_string();
    v[0] = format!("{}.{}", sec, nano);
    ans
}
*/

pub fn get_epochseconds() -> String {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    real.tv_sec().to_string()
}

pub fn get_epochrealtime(_: &mut Vec<String>) -> String {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    let epoch_realtime = format!("{}.{:06}", real.tv_sec(), real.tv_nsec() / 1000).to_string();
    epoch_realtime
}
