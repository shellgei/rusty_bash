//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::str::FromStr;
use ::time::Duration;
use nix::time;
use nix::time::ClockId;
use crate::core::database::variable::{Variable, DataType};

fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(now.tv_sec(), now.tv_nsec() as i32)
}

pub fn set_seconds() -> String {
    let offset = Duration::seconds(0);
    let adjusted = monotonic_time() - offset;
    format!("{}.{}", adjusted.whole_seconds(), adjusted.subsec_nanoseconds())
}

pub fn get_seconds(v: &mut Variable) -> String {
    if let DataType::Special(ref mut s) = v.value {
        let part: Vec<&str> = s.data.split('.').collect();
        let sec = i64::from_str(part[0]).unwrap();
        let nano = i32::from_str(part[1]).unwrap();
        let offset = Duration::new(sec, nano);
        let elapsed = monotonic_time() - offset;
        let ans = elapsed.whole_seconds().to_string();
        s.data = format!("{}.{}", sec, nano);
        //return DataType::Single(SingleData::from(elapsed.whole_seconds().to_string()));
        return ans;
    }
    //DataType::None
    "".to_string()
}

pub fn get_epochseconds(v: &mut Variable) -> String {
    if let DataType::Special(ref mut s) = v.value {
        let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
        let epoch_seconds = real.tv_sec().to_string();
       // DataType::Single(SingleData::from(epoch_seconds.to_string()))
        s.data = epoch_seconds.clone();
        return epoch_seconds;
    }
    "".to_string()
}

pub fn get_epochrealtime(v: &mut Variable) -> String {
    if let DataType::Special(ref mut s) = v.value {
        let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
        let epoch_realtime = format!("{}.{:06}", real.tv_sec(), real.tv_nsec() / 1000).to_string();
        //DataType::Single(SingleData::from(epoch_realtime))
        s.data = epoch_realtime.clone();
        return epoch_realtime;
    }
    "".to_string()
}
