//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::SimpleCommand;
use crate::elements::command;
use crate::elements::word::{Word, WordMode};
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};

pub fn set(
    com: &mut SimpleCommand,
    word: &Word,
    core: &mut ShellCore,
    feeder: &mut Feeder,
) -> Result<bool, ParseError> {
    com.continue_alias_check = false;
    let mut w = word.text.clone();
    if !core.replace_alias(&mut w) {
        return Ok(false);
    }

    com.continue_alias_check = w.ends_with(" ");
    let mut feeder_local = Feeder::new(&w);

    while SimpleCommand::eat_substitution(&mut feeder_local, com, core)? {
        command::eat_blank_with_comment(&mut feeder_local, core, &mut com.text);
    }

    loop {
        match Word::parse(&mut feeder_local, core, Some(WordMode::Alias)) {
            Ok(Some(w)) => {
                if w.text.starts_with("#") && com.words.is_empty() {
                    break;
                }
                com.text.push_str(&w.text);
                com.words.push(w);
            }
            _ => break,
        }
        command::eat_blank_with_comment(&mut feeder_local, core, &mut com.text);
    }

    if let Some(lst) = com.words.last() {
        if lst.text == "\\" {
            com.words.pop();
            feeder_local.replace(0, "\\");
        }
    }

    feeder.replace(0, &feeder_local.consume(feeder_local.len()));

    if com.words.is_empty() && com.substitutions.is_empty() {
        com.invalid_alias = true;
        return Ok(false);
    }

    Ok(true)
}
