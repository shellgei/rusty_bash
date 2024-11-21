//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

mod comparator;
mod extglob;
mod parser;

#[derive(Debug)]
pub enum Wildcard {
    Normal(String),
    Asterisk,
    Question,
    OneOf(Vec<char>),
    NotOneOf(Vec<char>),
    ExtGlob(char, Vec<String>),
}

pub fn parse_and_compare(word: &String, pattern: &str, extglob: bool) -> bool {
    let pat = parser::parse(pattern, extglob);
    compare(word, &pat)
}

pub fn compare(word: &String, pattern: &Vec<Wildcard>) -> bool {
    comparator::shave_word(word, pattern).iter().any(|c| c == "")
}

pub fn longest_match_length(word: &String, pattern: &Vec<Wildcard>) -> usize {
    word.len() - comparator::shave_word(word, pattern).iter()
                 .map(|c| c.len()).min().unwrap_or(word.len())
}

pub fn shortest_match_length(word: &String, pattern: &Vec<Wildcard>) -> usize {
    word.len() - comparator::shave_word(word, pattern).iter()
                 .map(|c| c.len()).max().unwrap_or(word.len())
}

pub fn parse(pattern: &str, extglob: bool) -> Vec<Wildcard> {
    parser::parse(pattern, extglob)
}
