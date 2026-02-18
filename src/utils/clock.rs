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
