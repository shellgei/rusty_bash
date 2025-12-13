//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct SRandomVar {
    rng: ChaCha20Rng,
    prev: String,
    flags: String,
}

impl Data for SRandomVar {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn _get_fmt_string(&self) -> String {
        self.prev.clone()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let rand = self.rng.next_u32();
        self.prev = rand.to_string();
        Ok(self.prev.clone())
    }

    fn len(&mut self) -> usize {
        self.prev.len()
    }

    fn is_special(&self) -> bool {
        true
    }

    fn has_flag(&mut self, flag: char) -> bool {
        flag == 'i'
    }

    fn get_flags(&mut self) -> String {
        self.flags.clone()
    }
}

impl SRandomVar {
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_os_rng(),
            prev: "".to_string(),
            flags: "i".to_string(),
        }
    }
}
