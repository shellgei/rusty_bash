//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use rand_chacha::ChaCha20Rng;
use rand::{RngCore, SeedableRng};
use crate::core::database::data::{Data, DataType};

fn gen_chacha20_u32(v: &mut Data) -> u32 {
    let seed = match &v.value {
        DataType::Single(s) => u64::from_str_radix(&s.data, 10).unwrap_or_else(|_| {
            ChaCha20Rng::from_entropy().next_u64()
        }),
        _ => {
            ChaCha20Rng::from_entropy().next_u64()
        }
    };

    ChaCha20Rng::seed_from_u64(seed).next_u32()
}

pub fn get_srandom(v: &mut Data) -> String {
    let rand = gen_chacha20_u32(v);
    v.set_data(rand.to_string());
    //v.value = DataType::Single(SingleData::from(rand.to_string()));
    //v.value.clone()
    rand.to_string()
}

pub fn get_random(v: &mut Data) -> String {
    let rand = gen_chacha20_u32(v) & 0x7FFF;
    v.set_data(rand.to_string());
    //v.value = DataType::Single(SingleData::from(rand.to_string()));
    //v.value.clone()
    rand.to_string()
}
