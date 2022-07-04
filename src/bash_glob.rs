//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;

#[derive(Debug)]
pub struct PatternElem {
    pub asterisk: bool,
    pub question: bool,
    pub inv: bool,
    pub chars: Vec<char>, 
    pub ranges: Vec<(char, char)>, 
}

pub fn judge(s: &String, pos: usize, pe: &PatternElem) -> Vec<usize> {
    let mut ans = vec!();
    if pe.asterisk {
        for n in pos..s.chars().count()+1 {
            ans.push(n);
        }

        return ans;
    }

    if pe.question {
        return vec!(pos+1);
    }

    if let Some(c) = s.chars().nth(pos) {
        let matched = pe.chars.iter().any(|ch| ch == &c) 
                      || pe.ranges.iter().any(|r| r.0 < c && c < r.1);

        if (pe.inv && ! matched) || (!pe.inv && matched) {
             ans.push(pos+1);
        }
    }

    ans
}

fn wildcard() -> PatternElem {
    PatternElem {
        asterisk: true,
        question: false,
        inv: false,
        chars: vec!(),
        ranges: vec!(),
    }
}

fn bracket(chs: &Vec<char>) -> PatternElem {
    let inv = chs[0] == '^' || chs[0] == '!';

    let chars = if inv {
        chs[1..].to_vec().clone()
    }else{
        chs.clone()
    };

    let mut chars2 = vec!();
    let mut ranges = vec!();
    for (i, c) in chars.iter().enumerate() {
        if c == &'-' && i >= 1 && i+1 < chars.len() {
            ranges.push((chars[i-1], chars[i+1]));
        }else{
            chars2.push(*c);
        }
    }

    return PatternElem {
        asterisk: false,
        question: false,
        inv: inv,
        chars: chars2,
        ranges: ranges,
    }
}

fn anychar() -> PatternElem {
    PatternElem {
        asterisk: false,
        question: true,
        inv: false,
        chars: vec!(),
        ranges: vec!(),
    }
}

fn simple_char(c: char) -> PatternElem {
    PatternElem {
        asterisk: false,
        question: false,
        inv: false,
        chars: vec!(c),
        ranges: vec!(),
    }
}

fn set_glob(glob: &String) -> Vec<PatternElem> {
    let mut ans = vec!();
    let mut pos = 0;
    let mut escaped = false;
    let mut in_brace = false;
    let mut bracket_str = vec!();

    loop {
        if glob.chars().count() == pos {
            break;
        }

        let ch = if let Some(c) = glob.chars().nth(pos) {
            c
        }else{
            panic!("Glob parse error");
        };

        pos += 1;

        if ! escaped && ch == '\\' {
            escaped = true;
            continue;
        }

        if escaped {
            ans.push(simple_char(ch));
        }else if ch == '*' {
            ans.push(wildcard());
        }else if ch == '?' {
            ans.push(anychar());
        }else if ch == '[' && ! in_brace {
            in_brace = true;
        }else if ch == ']' && in_brace {
            ans.push(bracket(&bracket_str));
            in_brace = false;
            bracket_str = vec!();
        }else if in_brace {
            bracket_str.push(ch);
        }else{
            ans.push(simple_char(ch));
        }

        escaped = false;
    }

    for ch in bracket_str {
        ans.push(simple_char(ch));
    }

    ans
}

pub fn glob_match(glob: &String, s: &String) -> bool {
    let pattern = set_glob(glob);
    let mut poss = HashSet::new();
    poss.insert(0);

    for pat in pattern {
        let mut poss_new = HashSet::new();
        for p in poss {
            for n in judge(s, p, &pat) {
                poss_new.insert(n);
            }
        }
        poss = poss_new;
        if poss.len() == 0 {
            break;
        }
    }

    ! poss.insert(s.chars().count())
}
