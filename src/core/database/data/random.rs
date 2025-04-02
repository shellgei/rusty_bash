//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::error::exec::ExecError;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use super::Data;

#[derive(Debug, Clone)]
pub struct RandomVar {
    rng: ChaCha20Rng,
    prev: String,
}

impl Data for RandomVar {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn print_body(&self) -> String { self.prev.clone() }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let rand = self.rng.next_u32() & 0x7FFF;
        self.prev = rand.to_string();
        Ok(self.prev.clone())
    }

    fn len(&mut self) -> usize {
        self.prev.len()
    }

    fn set_as_single(&mut self, value: &str) -> Result<(), ExecError> {
        let seed = u64::from_str_radix(&value, 10).unwrap_or(0);
        self.rng = ChaCha20Rng::seed_from_u64(seed);
        Ok(())
    }

    fn is_special(&self) -> bool {true}
}

impl RandomVar {
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(0),
            prev: "".to_string(),
        }
    }
}
