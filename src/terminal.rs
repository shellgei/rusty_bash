//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{InputError, ShellCore};
use std::io;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

pub struct Terminal {
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
        }
    }

    fn prompt_normal(&mut self, _core: &mut ShellCore) -> u16 {
        const PROMPT: &str = "🍣";
    
        print!("{} ", PROMPT);
        io::stdout().flush().unwrap();
    
        (UnicodeWidthStr::width(PROMPT) + 1 ) as u16 
    }

    pub fn read_line_normal(&mut self, core: &mut ShellCore) -> Result<String, InputError> {
        self.prompt_normal(core);
        Ok(String::new())
    }
}

