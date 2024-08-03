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

fn parse(subwords: &[Box<dyn Subword>], start: usize) -> Vec<usize> {
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
        if stack[i] == "," {
            delimiter_pos.insert(1, start+i);
        }else if stack[i] == "{" { // find an inner brace expcomma_posion
            stack[i..].iter_mut().for_each(|e| *e = "");
            return vec![];
        }
    }
    delimiter_pos
}

pub fn expand(_: &Vec<Box<dyn Subword>>, delimiters: &Vec<usize>) -> Vec<Word> {
    eprintln!("{:?}", delimiters);
    vec![]
}
