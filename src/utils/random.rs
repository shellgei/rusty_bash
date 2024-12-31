//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use rand_chacha::ChaCha20Rng;
use rand::{RngCore, SeedableRng};
//use crate::data::{Data, DataType};

fn gen_chacha20_u32(v: &mut Vec<String>) -> u32 {
    let seed = match v.len() {
        0 => ChaCha20Rng::from_entropy().next_u64(),
        _ => u64::from_str_radix(&v[0], 10)
            .unwrap_or_else(|_| { ChaCha20Rng::from_entropy().next_u64() }),
    };

    ChaCha20Rng::seed_from_u64(seed).next_u32()
}

pub fn get_srandom(v: &mut Vec<String>) -> String {
    if v.is_empty() {
        v.push("0.0".to_string());
    }
    let rand = gen_chacha20_u32(v);
    v[0] = rand.to_string();
    rand.to_string()
}

pub fn get_random(v: &mut Vec<String>) -> String {
    if v.is_empty() {
        v.push(ChaCha20Rng::from_entropy().next_u64().to_string())
    }
    let rand = gen_chacha20_u32(v) & 0x7FFF;
    v[0] = rand.to_string();
    rand.to_string()
}
