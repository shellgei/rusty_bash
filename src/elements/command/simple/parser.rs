//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, utils};
use super::{SimpleCommand};
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::utils::error::ParseError;

impl SimpleCommand {
    fn eat_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Substitution::parse(feeder, core) {
            ans.text += &s.text;
            match ans.permit_substitution_arg {
                true  => ans.substitutions_as_args.push(s),
                false => ans.substitutions.push(s),
            }
            true
        }else{
            false
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore) -> bool {
        let w = match Word::parse(feeder, core, false) {
            Some(w) => w,
            _       => {
                return false;
            },
        };

        if ans.words.is_empty() {
            if utils::reserved(&w.text) {
                return false;
            }else if w.text == "local" || w.text == "eval" {
                ans.permit_substitution_arg = true;
            }
        }

        if ans.words.is_empty() {
            if Self::set_alias(&w, &mut ans.words, &mut ans.text, core, feeder) {
                return true;
            }
        }

        ans.text += &w.text;
        ans.words.push(w);

        if ans.words.len() == 1 {
            ans.lineno = feeder.lineno;
        }
        true
    }

    fn set_alias(word: &Word, words: &mut Vec<Word>, text: &mut String,
                 core: &mut ShellCore, feeder: &mut Feeder) -> bool {
        let mut w = word.text.clone();
        if ! core.replace_alias(&mut w) {
            return false;
        }

        let mut feeder_local = Feeder::new(&mut w);
        loop {
            match Word::parse(&mut feeder_local, core, false) {
                Some(w) => {
                    text.push_str(&w.text);
                    words.push(w);
                },
                None    => break,
            }
            command::eat_blank_with_comment(&mut feeder_local, core, text);
        }

        if words.is_empty() {
            panic!("sush: alias: fatal alias");
        }

        feeder.replace(0, &feeder_local.consume(feeder_local.len()));
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        while Self::eat_substitution(feeder, &mut ans, core) {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            if ans.permit_substitution_arg 
            && Self::eat_substitution(feeder, &mut ans, core) {
                continue;
            }

            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! Self::eat_word(feeder, &mut ans, core) {
                break;
            }
        }

        if ans.substitutions.len() + ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
