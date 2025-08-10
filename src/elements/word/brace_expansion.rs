//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::single_quoted::SingleQuoted;
use crate::elements::subword::Subword;
use crate::elements::word::Word;

enum BraceType {
    Comma,
    Range(usize),
}

fn after_dollar(s: &str) -> bool {
    s == "$" || s == "$$"
}

fn num_to_subword(n: i128) -> Box<dyn Subword> {
    Box::new(SingleQuoted {
        text: format!("'{n}'"),
    })
}

fn ascii_to_subword(c: char) -> Box<dyn Subword> {
    let table = vec![
        "^?", "\\M-^@", "\\M-^A", "\\M-^B", "\\M-^C", "\\M-^D", "\\M-^E", "\\M-^F", "\\M-^G",
        "\\M-^H", "\\M-\t", "\\M-\n", "\\M-^K", "\\M-^L", "\\M-^M", "\\M-^N", "\\M-^O", "\\M-^P",
        "\\M-^Q", "\\M-^R", "\\M-^S", "\\M-^T", "\\M-^U", "\\M-^V", "\\M-^W", "\\M-^X", "\\M-^Y",
        "\\M-^Z", "\\M-^[", "\\M-^\\", "\\M-^]", "\\M-^^", "\\M-^_", " ", "¡",
    ];

    let n = c as usize;
    let text = if n >= 127 && n < 127 + table.len() {
        table[n - 127].to_string()
    } else {
        c.to_string()
    };

    Box::new(SingleQuoted {
        text: format!("'{text}'"),
    })
}

fn connect_minus(subwords: &mut Vec<Box<dyn Subword>>) {
    if subwords.len() < 2 {
        return;
    }

    let mut pos = vec![];
    for (i, sw) in subwords.iter().enumerate() {
        if sw.get_text() == "-" {
            pos.push(i);
        }
    }

    for i in pos {
        if i + 1 < subwords.len() {
            let mut num = true;
            for ch in subwords[i + 1].get_text().chars() {
                if !ch.is_ascii_digit() {
                    num = false;
                    break;
                }
            }

            if !num {
                continue;
            }

            subwords[i] = Default::default();
            let m_str = "-".to_owned() + subwords[i + 1].get_text();
            subwords[i + 1] = From::from(&m_str);
        }
    }

    subwords.retain(|e| !e.get_text().is_empty());
}

pub fn eval(word: &mut Word, compat_bash: bool) -> Vec<Word> {
    invalidate_brace(&mut word.subwords);
    connect_minus(&mut word.subwords);

    let mut skip_until = 0;
    for i in word.scan_pos("{") {
        if i < skip_until {
            //ブレース展開の終わりまで処理をスキップ
            continue;
        }

        if let Some(d) = parse(&word.subwords[i..]) {
            let shift_d: Vec<usize> = d.0.iter().map(|e| e + i).collect();

            if i > 0 && after_dollar(word.subwords[i - 1].get_text()) {
                skip_until = *shift_d.last().unwrap();
                continue;
            }

            return match d.1 {
                BraceType::Comma => expand_comma_brace(&word.subwords, &shift_d, compat_bash),
                BraceType::Range(n) => {
                    expand_range_brace(&mut word.subwords, &shift_d, n, compat_bash)
                }
            };
        }
    }

    vec![word.clone()]
}

fn invalidate_brace(subwords: &mut Vec<Box<dyn Subword>>) {
    if subwords.len() < 2 {
        return;
    }

    if subwords[0].get_text() == "{" && subwords[1].get_text() == "}" {
        subwords.remove(1);
        subwords[0].set_text("{}");
    }
}

fn parse(subwords: &[Box<dyn Subword>]) -> Option<(Vec<usize>, BraceType)> {
    let mut stack = vec![];
    for sw in subwords {
        stack.push(Some(sw.get_text()));
        if sw.get_text() == "}" {
            if let Some(found) = get_delimiters(&mut stack) {
                return Some(found);
            }
        }
    }
    None
}

fn get_delimiters(stack: &mut [Option<&str>]) -> Option<(Vec<usize>, BraceType)> {
    let mut comma_pos = vec![];
    let mut period_pos = vec![];

    for i in (1..stack.len() - 1).rev() {
        if stack[i] == Some(",") {
            comma_pos.push(i);
        } else if stack[i] == Some(".") {
            period_pos.push(i);
        } else if stack[i] == Some("{") {
            // find an inner brace expcomma_posion
            stack[i..].iter_mut().for_each(|e| *e = None);
            return None;
        }
    }

    if !comma_pos.is_empty() {
        comma_pos.reverse();
        comma_pos.insert(0, 0); // add "{" pos
        comma_pos.push(stack.len() - 1); // add "}" pos
        return Some((comma_pos, BraceType::Comma));
    }

    if period_pos.len() == 2 && period_pos[0] == period_pos[1] + 1 {
        period_pos.reverse();
        period_pos.insert(0, 0);
        period_pos.push(stack.len() - 1);
        return Some((period_pos, BraceType::Range(2)));
    }

    if period_pos.len() == 4
        && period_pos[0] == period_pos[1] + 1
        && period_pos[2] == period_pos[3] + 1
    {
        period_pos.reverse();
        period_pos.insert(0, 0);
        period_pos.push(stack.len() - 1);
        return Some((period_pos, BraceType::Range(3)));
    }

    None
}

fn comma_brace_to_subwords(
    subwords: &[Box<dyn Subword>],
    delimiters: &[usize],
) -> Vec<Vec<Box<dyn Subword>>> {
    let mut ans = vec![];
    let mut from = delimiters[0] + 1;
    for to in &delimiters[1..] {
        ans.push(subwords[from..*to].to_vec());
        from = *to + 1;
    }
    ans
}

fn expand_comma_brace(
    subwords: &[Box<dyn Subword>],
    delimiters: &[usize],
    compat_bash: bool,
) -> Vec<Word> {
    let left = subwords[..delimiters[0]].to_vec();
    let mut right = subwords[(delimiters.last().unwrap() + 1)..].to_vec();
    invalidate_brace(&mut right);

    let sws = comma_brace_to_subwords(subwords, delimiters);
    subword_sets_to_words(&sws, &left, &right, compat_bash)
}

fn expand_range_brace(
    subwords: &mut Vec<Box<dyn Subword>>,
    delimiters: &[usize],
    operand_num: usize,
    compat_bash: bool,
) -> Vec<Word> {
    let start_wrap = subwords[delimiters[0] + 1].make_unquoted_string(); // right of {
    let end_wrap = subwords[delimiters[2] + 1].make_unquoted_string(); // left of }

    let (start, end) = match (start_wrap, end_wrap) {
        (Some(s), Some(e)) => (s, e),
        _ => return subwords_to_word(subwords),
    };

    let mut skip_num = match operand_num {
        2 => 1,
        3 => {
            let skip = subwords[delimiters[4] + 1].get_text();
            match skip.parse::<i32>() {
                Ok(n) => n.unsigned_abs() as usize,
                _ => return subwords_to_word(subwords),
            }
        }
        _ => return subwords_to_word(subwords),
    };
    skip_num = std::cmp::max(skip_num, 1);

    let mut series = gen_nums(&start, &end, skip_num);
    if series.is_empty() {
        series = gen_chars(&start, &end, skip_num, compat_bash);
    }
    if series.is_empty() {
        return subwords_to_word(subwords);
    }

    if compat_bash {
        for e in series.iter_mut() {
            if e.get_text() == "'\\'" {
                *e = Box::new(SingleQuoted {
                    text: "''".to_string(),
                });
            }
        }
    }

    let mut series2 = vec![];
    for e in series {
        series2.push(vec![e]);
    }

    let left = &subwords[..delimiters[0]];
    let mut right = subwords[(delimiters.last().unwrap() + 1)..].to_vec();
    invalidate_brace(&mut right);

    subword_sets_to_words(&series2, left, &right, compat_bash)
}

fn gen_nums(start: &str, end: &str, skip: usize) -> Vec<Box<dyn Subword>> {
    let (start_num, end_num) = match (start.parse::<i128>(), end.parse::<i128>()) {
        (Ok(s), Ok(e)) => (s, e),
        _ => return vec![],
    };

    let min = std::cmp::min(start_num, end_num);
    let max = std::cmp::max(start_num, end_num);

    let mut ans: Vec<Box<dyn Subword>> = (min..(max + 1)).map(|n| num_to_subword(n)).collect();
    if start_num > end_num {
        ans.reverse();
    }
    ans.into_iter()
        .enumerate()
        .filter(|e| e.0 % skip == 0)
        .map(|e| e.1)
        .collect()
}

fn gen_chars(start: &str, end: &str, skip: usize, compat_bash: bool) -> Vec<Box<dyn Subword>> {
    let (start_num, end_num) = match (start.chars().next(), end.chars().next()) {
        (Some(s), Some(e)) => (s, e),
        _ => return vec![],
    };

    if start.chars().count() > 1 || end.chars().count() > 1 {
        return vec![];
    }

    let min = std::cmp::min(start_num, end_num);
    let max = std::cmp::max(start_num, end_num);

    if compat_bash {
        if min.is_ascii_digit() && !max.is_ascii_digit() {
            return vec![];
        }
        if max.is_ascii_digit() && !min.is_ascii_digit() {
            return vec![];
        }
    }

    let mut ans: Vec<Box<dyn Subword>> = (min..max).map(|n| ascii_to_subword(n)).collect();
    ans.push(ascii_to_subword(max));
    if start_num > end_num {
        ans.reverse();
    }

    ans.into_iter()
        .enumerate()
        .filter(|e| e.0 % skip == 0)
        .map(|e| e.1)
        .collect()
}

fn subword_sets_to_words(
    series: &[Vec<Box<dyn Subword>>],
    left: &[Box<dyn Subword>],
    right: &[Box<dyn Subword>],
    compat_bash: bool,
) -> Vec<Word> {
    let mut ws = vec![];
    for sws in series {
        let mut w = Word::default();
        w.subwords = [left, sws, right].concat();
        w.text = w.subwords.iter().map(|s| s.get_text()).collect();
        ws.push(w);
    }

    let mut ans = vec![];
    for w in ws.iter_mut() {
        ans.append(&mut eval(w, compat_bash));
    }
    ans
}

fn subwords_to_word(subwords: &[Box<dyn Subword>]) -> Vec<Word> {
    let mut w = Word::default();
    w.subwords = subwords.to_vec();
    w.text = w.subwords.iter().map(|s| s.get_text()).collect();
    vec![w]
}
