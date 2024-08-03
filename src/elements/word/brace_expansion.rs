//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use super::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    for i in open_brace_pos(word) {
        let d = parse(&word.subwords[i..], i);
        if d.len() > 2 {
            return expand(&word.subwords, &d);
        }
    }
    vec![word.clone()]
}

fn open_brace_pos(w: &Word) -> Vec<usize> {
    w.subwords.iter()
        .enumerate()
        .filter(|e| e.1.get_text() == "{")
        .map(|e| e.0)
        .collect()
}

pub fn parse(subwords: &[Box<dyn Subword>], start: usize) -> Vec<usize> {
    let mut stack = vec![];
    for sw in subwords {
        stack.push(sw.get_text());
        if sw.get_text() == "}" {
            let ds = get_delimiters(&mut stack, start);
            if ds.len() > 2 {
                return ds;
            }
        }
    }
    vec![]
}

fn get_delimiters(stack: &mut Vec<&str>, start: usize) -> Vec<usize> {
    let mut delimiter_pos = vec![start, stack.len()-1+start];
    for i in (1..stack.len()-1).rev() {
        match stack[i] {
            "," => delimiter_pos.insert(1, start+i),
            "{" => { // find an inner brace expdelimiter_posion
                stack[i..].iter_mut().for_each(|e| *e = "");
                return vec![];
            },
            _   => {},
        }
    }
    delimiter_pos
}

pub fn expand(subwords: &Vec<Box<dyn Subword>>, delimiters: &Vec<usize>) -> Vec<Word> {
    let left = &subwords[..delimiters[0]];
    let right = &subwords[(delimiters.last().unwrap()+1)..];

    let mut ans = vec![];
    let mut from = delimiters[0] + 1;
    for to in &delimiters[1..] {
        let mut w = Word::new();
        w.subwords = [ left, &subwords[from..*to], right ].concat();
        w.text = w.subwords.iter().map(|s| s.get_text()).collect();
        ans.append(&mut eval(&mut w));
        from = *to + 1;
    }
    ans
}
