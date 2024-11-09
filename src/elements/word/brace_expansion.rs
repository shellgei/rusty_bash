//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;
use super::Word;

fn after_dollar(s: &str) -> bool {
    s == "$" || s == "$$"
}

pub fn eval(word: &mut Word) -> Vec<Word> {
    invalidate_brace(&mut word.subwords);

    let mut skip_until = 0;
    for i in word.scan_pos("{") {
        if i < skip_until { //ブレース展開の終わりまで処理をスキップ
            continue;
        }

        let d = parse(&word.subwords[i..], i);
        if d.len() <= 2 {
            continue;
        }

        match i > 0 && after_dollar(word.subwords[i-1].get_text()) {
            true  => skip_until = *d.last().unwrap(),
            false => return expand(&word.subwords, &d),
        }
    }
    vec![word.clone()]
}

fn invalidate_brace(subwords: &mut Vec<Box<dyn Subword>>) {
    if subwords.len() < 2 {
        return;
    }

    if subwords[0].get_text() == "{"
    && subwords[1].get_text() == "}" {
        subwords.remove(1);
        subwords[0].set_text("{}");
    }
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

fn expand(subwords: &Vec<Box<dyn Subword>>, delimiters: &Vec<usize>) -> Vec<Word> {
    let left = &subwords[..delimiters[0]];
    let mut right = subwords[(delimiters.last().unwrap()+1)..].to_vec();
    invalidate_brace(&mut right);

    let mut ans = vec![];
    for i in 0..(delimiters.len()-1) {
        let center = &subwords[ (delimiters[i]+1)..delimiters[i+1] ];
        let mut w = Word::concat_new(&[ left, &center, &right ] );
        ans.append(&mut eval(&mut w));
    }
    ans
}
