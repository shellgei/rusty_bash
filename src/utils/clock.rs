//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use nix::time;
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
