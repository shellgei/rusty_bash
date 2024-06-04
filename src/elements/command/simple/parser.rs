//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{SimpleCommand};
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;

fn reserved(w: &str) -> bool {
    match w {
        "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" | "case" => true,
        _ => false,
    }
}

impl SimpleCommand {
    fn new() -> SimpleCommand {
        SimpleCommand {
            text: String::new(),
            substitutions: vec![],
            evaluated_subs: vec![],
            words: vec![],
            args: vec![],
            redirects: vec![],
            force_fork: false,
            substitutions_as_args: vec![],
            permit_substitution_arg: false,
        }
    }

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
        let w = match Word::parse(feeder, core) {
            Some(w) => w,
            _       => {
                return false;
            },
        };

        if ans.words.len() == 0 {
            if reserved(&w.text) {
                return false;
            }else if w.text == "local" {
                ans.permit_substitution_arg = true;
            }
        }
        ans.text += &w.text;
        ans.words.push(w);
        Self::set_alias(&mut ans.words, core, feeder);

        true
    }

    fn set_alias(words: &mut Vec<Word>, core: &mut ShellCore, feeder: &mut Feeder) {
        if words.len() == 0 {
            return;
        }

        let mut w = words[0].text.clone();
        core.data.replace_alias(&mut w);
        let mut feeder_local = Feeder::new(&mut w);
        let mut alias_words = vec![];
        let mut dummy = String::new();
        loop {
            match Word::parse(&mut feeder_local, core) {
                Some(w) => alias_words.push(w),
                None    => break,
            }
            command::eat_blank_with_comment(&mut feeder_local, core, &mut dummy);
        }

        if alias_words.len() == 0 {
            return;
        }

        feeder.replace(0, &feeder_local.consume(feeder_local.len()));

        words.remove(0);
        alias_words.append(words);
        *words = alias_words;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = Self::new();
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
            //Self::set_alias(&mut ans.words, core, feeder);
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
