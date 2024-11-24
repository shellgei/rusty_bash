//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
pub enum GlobElem {
    Normal(String),
    Asterisk,
    Question,
    OneOf(Vec<char>),
    NotOneOf(Vec<char>),
}

pub fn parse(pattern: &str) -> Vec<GlobElem> {
    let pattern = pattern.to_string();
    let mut remaining = pattern.to_string();
    let mut ans = vec![];

    while remaining.len() > 0 {
        if remaining.starts_with("*") {
            consume(&mut remaining, 1); 
            ans.push( GlobElem::Asterisk );
            continue;
        }else if remaining.starts_with("?") {
            consume(&mut remaining, 1); 
            ans.push( GlobElem::Question );
            continue;
        }

        let s = consume(&mut remaining, 1);
        ans.push( GlobElem::Normal(s) );
    }

    ans
}

fn consume(remaining: &mut String, cutpos: usize) -> String {
    let cut = remaining[0..cutpos].to_string();
    *remaining = remaining.split_off(cutpos);

    cut
}
