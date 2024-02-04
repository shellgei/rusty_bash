//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use crate::elements::word::Word;

pub fn eval(word: &mut Word) -> Vec<Word> {
    let mut start_pos = open_brace_pos(word);
    if word.text.starts_with("{}") {
        start_pos.remove(0);
    }

    for i in start_pos {
        if let Some(d) = parse(&word.subwords[i..]) {
            let shift_d = d.iter().map(|e| e+i).collect();
            return expand(&word.subwords, &shift_d);
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

pub fn parse(subwords: &[Box<dyn Subword>]) -> Option<Vec<usize>> {
    let mut stack = vec![];
    for sw in subwords {
        stack.push(Some(sw.get_text()));
        if sw.get_text() == "}" {
            match get_delimiters(&mut stack) {
                Some(ds) => return Some(ds),
                _        => {},
            }
        }
    }
    None
}

fn get_delimiters(stack: &mut Vec<Option<&str>>) -> Option<Vec<usize>> {
    let mut comma_pos = vec![];
    for i in (1..stack.len()-1).rev() {
        if stack[i] == Some(",") {
            comma_pos.push(i);
        }else if stack[i] == Some("{") { // find an inner brace expcomma_posion
            stack[i..].iter_mut().for_each(|e| *e = None);
            return None;
        }
    }

    if comma_pos.len() > 0 {
        comma_pos.reverse();
        comma_pos.insert(0, 0); // add "{" pos
        comma_pos.push(stack.len()-1); // add "}" pos
        Some(comma_pos)
    }else{
        None
    }
}

pub fn expand(subwords: &Vec<Box<dyn Subword>>, delimiters: &Vec<usize>) -> Vec<Word> {
    let left_subs = &subwords[..delimiters[0]];
    let right_subs = &subwords[(delimiters.last().unwrap()+1)..];

    let mut ans = vec![];
    let mut from = delimiters[0] + 1;
    for to in &delimiters[1..] {
        let mut w = Word::new();
        w.extend(left_subs)
         .extend(&subwords[from..*to])
         .extend(&right_subs);
        ans.append(&mut eval(&mut w));
        from = *to + 1;
    }
    ans
}