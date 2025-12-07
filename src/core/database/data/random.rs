//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::Data;
use crate::error::exec::ExecError;
use rand_chacha::rand_core::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct RandomVar {
    rng: ChaCha20Rng,
    flags: String, 
}

impl Data for RandomVar {
    fn boxed_clone(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }

    fn get_print_string_fix(&self) -> String {
        "".to_string()
    }

    fn get_as_single(&mut self) -> Result<String, ExecError> {
        let rand = self.rng.next_u32() & 0x7FFF;
        //self.prev = rand.to_string();
        Ok(rand.to_string())
    }

    fn len(&mut self) -> usize {
        self.get_as_single().unwrap().len()
    }

    fn set_as_single(&mut self, name: &str, value: &str) -> Result<(), ExecError> {
        self.readonly_check(name)?;

        let seed = value.parse::<u64>().unwrap_or(0);
        self.rng = ChaCha20Rng::seed_from_u64(seed + 4011); //4011: for bash test
        Ok(())
    }

    fn is_special(&self) -> bool {
        true
    }

    fn set_flag(&mut self, flag: char) {
        if ! self.flags.contains(flag) {
            self.flags.push(flag);
        }
    }

    fn unset_flag(&mut self, flag: char) {
        self.flags.retain(|e| e != flag);
    }

    fn has_flag(&mut self, flag: char) -> bool {
        self.flags.contains(flag)
    }
}

impl RandomVar {
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(0),
            flags: "i".to_string(),
        }
    }
}
