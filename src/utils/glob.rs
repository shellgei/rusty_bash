//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod comparator;
mod extglob;
mod parser;

#[derive(Debug)]
pub enum GlobElem {
    Normal(String),
    Symbol(char),
    OneOf(bool, Vec<MetaChar>),
    ExtGlob(char, Vec<String>),
}

#[derive(Debug)]
pub enum MetaChar {
    Normal(char),
    Range(char, char),
    CharClass(String),
}

pub fn parse_and_compare(word: &str, pattern: &str, extglob: bool) -> bool {
    let pat = parser::parse(pattern, extglob);
    compare(word, &pat)
}

pub fn compare(word: &str, pattern: &[GlobElem]) -> bool {
    comparator::shave_word(word, pattern)
        .iter()
        .any(|c| c.is_empty())
}

pub fn longest_match_length(word: &str, pattern: &[GlobElem]) -> usize {
    word.len()
        - comparator::shave_word(word, pattern)
            .iter()
            .map(|c| c.len())
            .min()
            .unwrap_or(word.len())
}

pub fn shortest_match_length(word: &str, pattern: &[GlobElem]) -> usize {
    word.len()
        - comparator::shave_word(word, pattern)
            .iter()
            .map(|c| c.len())
            .max()
            .unwrap_or(word.len())
}

pub fn parse(pattern: &str, extglob: bool) -> Vec<GlobElem> {
    parser::parse(pattern, extglob)
}
