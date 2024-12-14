//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::str::FromStr;
use ::time::Duration;
use nix::time;
use nix::time::ClockId;
use crate::core::data::variable::{Variable, Value};

fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(now.tv_sec(), now.tv_nsec() as i32)
}

pub fn set_seconds(_v: &mut Variable, var: &str) -> Value {
    let offset = Duration::seconds(i64::from_str(var).unwrap_or(0));
    let adjusted = monotonic_time() - offset;
    Value::EvaluatedSingle(format!("{}.{}", adjusted.whole_seconds(), adjusted.subsec_nanoseconds()))
}

pub fn get_seconds(v: &mut Variable) -> Value {
    if let Value::EvaluatedSingle(ref s) = v.value {
        let part: Vec<&str> = s.split('.').collect();
        let sec = i64::from_str(part[0]).unwrap();
        let nano = i32::from_str(part[1]).unwrap();
        let offset = Duration::new(sec, nano);
        let elapsed = monotonic_time() - offset;
        return Value::EvaluatedSingle(elapsed.whole_seconds().to_string());
    }
    Value::None
}

pub fn get_epochseconds(_v: &mut Variable) -> Value {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    let epoch_seconds = real.tv_sec();
    Value::EvaluatedSingle(epoch_seconds.to_string())
}

pub fn get_epochrealtime(_v: &mut Variable) -> Value {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    let epoch_realtime = format!("{}.{:06}", real.tv_sec(), real.tv_nsec() / 1000);
    Value::EvaluatedSingle(epoch_realtime.to_string())
}
