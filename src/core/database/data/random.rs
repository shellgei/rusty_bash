//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, ExecError};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

#[derive(Debug, Clone)]
pub struct RandomVar {
    rng: ChaCha20Rng,
    flags: String,
}

impl Data for RandomVar {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let rand = self.rng.next_u32() & 0x7FFF;
        Ok(rand.to_string())
    }

    fn get_flags(&self) -> &str {
        &self.flags
    }
}

impl RandomVar {
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_os_rng(),
            flags: "i".to_string(),
        }
    }
}
