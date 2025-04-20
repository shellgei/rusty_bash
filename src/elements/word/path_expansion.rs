//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::core::options::Options;
use crate::elements::subword::Subword;
use crate::elements::word::Word;
use crate::utils::directory;
use super::subword::simple::SimpleSubword;

pub fn eval(word: &mut Word, shopts: &Options) -> Vec<Word> {
    let globstr = word.make_glob_string();
    if no_glob_symbol(&globstr) {
        return vec![word.clone()];
    }

    let paths = expand(&globstr, shopts);
    if paths.is_empty() {
        if shopts.query("nullglob") {
            return vec![Word::from(&String::new())];
        }
        return vec![word.clone()];
    }

    let subwd = |path| Box::new(SimpleSubword{ text: path });
    let wd = |path| Word::from( subwd(path) as Box::<dyn Subword>);
    paths.iter().map(|p| wd(p.to_string())).collect()
}

fn no_glob_symbol(pattern: &str) -> bool {
    "*?@+![".chars().all(|c| ! pattern.contains(c))
}

pub fn expand(pattern: &str, shopts: &Options) -> Vec<String> {
    let mut paths = vec!["".to_string()];

    for dir_glob in pattern.split("/") {
        let mut tmp = paths.iter()
                .map(|c| directory::glob(&c, &dir_glob, shopts) )
                .collect::<Vec<Vec<String>>>()
                .concat();

        if dir_glob == "**" && shopts.query("globstar") {
            tmp.append(&mut paths);
        }
        paths = tmp;

        paths.sort();
        paths.dedup();
    }

    paths.iter_mut().for_each(|e| {e.pop();} );

    if shopts.query("globstar") {
        if let Some(ptn) = pattern.strip_suffix("/**") {
            paths.iter_mut().for_each(|p| if p == ptn {*p += "/";} );
        }
    }

    paths.sort();
    paths
}
