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
    let globstar = shopts.query("globstar");
    let mut exist_globstar = false;
    let mut compat_bash = false;
    let mut last_path = "".to_string();

    for dir_glob in pattern.split("/") {
        paths.sort();
        paths.dedup();

        if exist_globstar {
            if dir_glob != "**" {
                if last_path == "" {
                    last_path = dir_glob.to_owned() + "/";
                }
                compat_bash = true;
            }
        }else if dir_glob == "**" {
            exist_globstar = true;
        }
        
        let mut tmp = paths.iter()
                .map(|c| directory::glob(&c, &dir_glob, shopts) )
                .collect::<Vec<Vec<String>>>()
                .concat();

        if dir_glob == "**" && globstar {
            paths.retain(|p| p.ends_with(&last_path));
            tmp.append(&mut paths);
        }

        paths = tmp;
    }

    paths.iter_mut().for_each(|e| {e.pop();} );

    if shopts.query("globstar") {
        if let Some(ptn) = pattern.strip_suffix("/**") {
            paths.iter_mut().for_each(|p| if p == ptn {*p += "/";} );
        }
    }

    paths.sort();
    if ! compat_bash {
        paths.dedup();
    }
    paths
}
