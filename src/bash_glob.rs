//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;

#[derive(Debug)]
pub struct PatternElem {
    pub all: bool,
    pub chars: Vec<char>, 
    pub ranges: Vec<(char, char)>, 
}

pub fn judge(s: &String, pos: usize, pe: &PatternElem) -> Vec<usize> {
    let mut ans = vec!();
    if pe.all {
        for n in pos..s.chars().count()+1 {
            ans.push(n);
        }

        return ans;
    }

    let mut p = pos;

    if let Some(c) = s.chars().nth(p) {
        if pe.chars.iter().any(|ch| ch == &c) {
            ans.push(p+1);
        }else{
            return vec!();
        }
    }

    ans
}

fn wildcard() -> PatternElem {
    PatternElem {
        all: true,
        chars: vec!(),
        ranges: vec!(),
    }
}

fn simple_char(c: char) -> PatternElem {
    PatternElem {
        all: false,
        chars: vec!(c),
        ranges: vec!(),
    }
}

pub fn glob_set(glob: &String) -> Vec<PatternElem> {
    let mut ans = vec!();
    let mut pos = 0;
    loop {
        if glob.chars().count() == pos {
            return ans;
        }

        let ch = if let Some(c) = glob.chars().nth(pos) {
            c
        }else{
            panic!("Glob parse error");
        };
        pos += 1;

        if ch == '*' {
            ans.push(wildcard());
        }else{
            ans.push(simple_char(ch));
        }
    }
}

pub fn glob_match(glob: &String, s: &String) -> bool {
    let pattern = glob_set(glob);
    let mut poss = HashSet::new();
    poss.insert(0);

    eprintln!("{}\n{}", glob, s);

    for pat in pattern {
        let mut poss_new = HashSet::new();
        for p in poss {
            for n in judge(s, p, &pat) {
                poss_new.insert(n);
            }
        }
        poss = poss_new;
        eprintln!("{:?}", poss);
    }

    eprintln!("RES: {:?}, LEN: {}", poss, s.len());
    ! poss.insert(s.len())
}
