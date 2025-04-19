//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod helper;
mod completion;
mod prompt;

use crate::ShellCore;
use crate::error::input::InputError;
use std::sync::atomic::Ordering::Relaxed;
use rustyline::{Editor, Config, EditMode, CompletionType};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use std::io::{stdout, Write};

use helper::SushHelper;
use prompt::{make_prompt_string, parse_visible_prompt, oct_to_hex_in_str};

pub struct Terminal {
    rl: Editor<SushHelper, DefaultHistory>,
}

impl Terminal {
    pub fn new(core: &mut ShellCore) -> Terminal {
        let config = Config::builder()
            .edit_mode(EditMode::Emacs)
            .auto_add_history(true)
            .color_mode(rustyline::ColorMode::Enabled)
            .completion_type(CompletionType::List)
            .build();
        let mut rl: Editor<SushHelper, DefaultHistory> = Editor::with_config(config).unwrap();
        let helper = SushHelper::new();
        rl.set_helper(Some(helper));
        if let Ok(history_file) = core.db.get_param("HISTFILE") {
            if !history_file.is_empty() {
                let path = std::path::PathBuf::from(&history_file);
                let _ = rl.load_history(&path);
            }
        }
        for entry in core.history.iter().rev() {
            let _ = rl.add_history_entry(entry);
        }
        Terminal { rl }
    }

    pub fn read_line(&mut self, core: &mut ShellCore, prompt: &str) -> Result<String, InputError> {
        let raw = core.db.get_param(prompt).unwrap_or_default();
        let replaced = make_prompt_string(&raw);
        let ansi = oct_to_hex_in_str(&replaced);
        let (display, hidden) = parse_visible_prompt(&ansi);
        // output hidden parts for ANSI sequences
        print!("{}", hidden);
        stdout().flush().unwrap();
        if let Some(helper) = self.rl.helper_mut() {
            helper.set_prompt(display.clone());
        }
        match self.rl.readline(&display) {
            Ok(line) => Ok(line),
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                core.sigint.store(true, Relaxed);
                Err(InputError::Interrupt)
            },
            Err(ReadlineError::Eof) => Err(InputError::Eof),
            Err(_) => Err(InputError::Interrupt),
        }
    }

    pub fn save_history(&mut self, path: &std::path::Path) {
        let _ = self.rl.save_history(path);
    }
}
