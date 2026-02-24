//SPDXFileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::{Data, ExecError};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

#[derive(Debug, Clone)]
pub struct SRandomVar {
    rng: ChaCha20Rng,
    flags: String,
}

impl Data for SRandomVar {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_fmt_string(&mut self) -> String {
        self.get_as_single().unwrap()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let rand = self.rng.next_u32();
        Ok(rand.to_string())
    }

    fn set_as_single(&mut self, name: &str, _: &str) -> Result<(), ExecError> {
        self.readonly_check(name)
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn get_flags(&self) -> &str {
        &self.flags
    }
}

impl SRandomVar {
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_os_rng(),
            flags: "i".to_string(),
        }
    }
}
