//SPDX-FileCopyrightText: 2025 Hugo Fortin
//SPDX-License-Identifier: BSD-3-Clause

///// Custom and ultra-light Regex implementation /////

use std::iter::Peekable;
use std::str::Chars;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;

#[derive(Debug, Clone)]
pub enum RegexAst {
    Literal(char),
    AnyChar,
    StartAnchor,
    EndAnchor,
    ZeroOrMore(Box<RegexAst>),
    OneOrMore(Box<RegexAst>),
    ZeroOrOne(Box<RegexAst>),
    Sequence(Vec<RegexAst>),
    Alternate(Vec<RegexAst>),
    Group(Box<RegexAst>),
    CharClass(Vec<CharSetItem>),
    NegatedCharClass(Vec<CharSetItem>),
}

#[derive(Debug, Clone)]
pub enum CharSetItem {
    Single(char),
    Range(char, char),
}

fn regex_match(pattern: &str, text: &str) -> bool {
    match parse(pattern) {
        Ok(ast) => match_here(&ast, text),
        Err(_) => false,
    }
}

fn parse(pattern: &str) -> Result<RegexAst, ExecError> {
    let mut chars = pattern.chars().peekable();
    parse_expr(&mut chars)
}

pub fn parse_regex(pattern: &str) -> Result<RegexAst, ExecError> {
    parse(pattern)
}

fn parse_expr(chars: &mut Peekable<Chars>) -> Result<RegexAst, ExecError> {
    let mut terms = Vec::new();
    let mut branches = Vec::new();

    while let Some(&c) = chars.peek() {
        match c {
            ')' => break,
            '|' => {
                chars.next();
                if !terms.is_empty() {
                    branches.push(RegexAst::Sequence(terms));
                    terms = Vec::new();
                } else {
                    branches.push(RegexAst::Sequence(vec![]));
                }
            }
            _ => terms.push(parse_term(chars)?),
        }
    }

    if !branches.is_empty() {
        branches.push(RegexAst::Sequence(terms));
        return Ok(RegexAst::Alternate(branches));
    }

    Ok(RegexAst::Sequence(terms))
}

fn parse_term(chars: &mut Peekable<Chars>) -> Result<RegexAst, ExecError> {
    let mut atom = match chars.next() {
        Some('^') => RegexAst::StartAnchor,
        Some('$') => RegexAst::EndAnchor,
        Some('.') => RegexAst::AnyChar,
        Some('(') => {
            let inner = parse_expr(chars)?;
            if chars.next() != Some(')') {
                return Err(ExecError::from(ParseError::Regex("Unclosed ( group".to_string())));
            }
            RegexAst::Group(Box::new(inner))
        }
        Some('[') => parse_char_class(chars)?,
        Some('\\') => {
            let c = chars.next().ok_or_else(|| ExecError::from(ParseError::Regex("Backslash without character".to_string())))?;
            RegexAst::Literal(c)
        }
        Some(c) => RegexAst::Literal(c),
        None => return Err(ExecError::from(ParseError::Regex("Unexpected end of pattern".to_string()))),
    };

    while let Some(&next) = chars.peek() {
        atom = match next {
            '*' => {
                chars.next();
                RegexAst::ZeroOrMore(Box::new(atom))
            }
            '+' => {
                chars.next();
                RegexAst::OneOrMore(Box::new(atom))
            }
            '?' => {
                chars.next();
                RegexAst::ZeroOrOne(Box::new(atom))
            }
            _ => break,
        };
    }

    Ok(atom)
}

pub fn glob_to_regex(glob: &str) -> String {
    let mut regex = String::from("^");
    for c in glob.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' | '#' | ' ' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }
    regex.push('$');
    regex
}

fn parse_char_class(chars: &mut Peekable<Chars>) -> Result<RegexAst, ExecError> {
    let mut negate = false;
    let mut items = Vec::new();

    if let Some(&c) = chars.peek() {
        if c == '^' {
            negate = true;
            chars.next();
        }
    }

    while let Some(c) = chars.next() {
        if c == ']' && !items.is_empty() {
            break;
        }

        if c == '-' {
            if let Some(CharSetItem::Single(start)) = items.pop() {
                if let Some(&end) = chars.peek() {
                    chars.next();
                    items.push(CharSetItem::Range(start, end));
                } else {
                    return Err(ExecError::from(ParseError::Regex("Incomplete character range".to_string())));
                }
            } else {
                items.push(CharSetItem::Single('-'));
            }
        } else {
            items.push(CharSetItem::Single(c));
        }
    }

    if items.is_empty() {
        return Err(ExecError::from(ParseError::Regex("Empty character class".to_string())));
    }

    Ok(if negate {
        RegexAst::NegatedCharClass(items)
    } else {
        RegexAst::CharClass(items)
    })
}

pub fn match_here(ast: &RegexAst, text: &str) -> bool {
    match ast {
        RegexAst::Sequence(seq) => match_sequence(seq, text),
        RegexAst::Alternate(choices) => choices.iter().any(|a| match_here(a, text)),
        _ => match_one(ast, text).map_or(false, |rest| rest.is_empty()),
    }
}

fn match_sequence(seq: &[RegexAst], text: &str) -> bool {
    let anchored_start = matches!(seq.first(), Some(RegexAst::StartAnchor));
    let anchored_end = matches!(seq.last(), Some(RegexAst::EndAnchor));

    let core = if anchored_start {
        &seq[1..seq.len() - if anchored_end { 1 } else { 0 }]
    } else {
        seq
    };

    if anchored_start {
        if anchored_end {
            return match_sequence_strict(core, text);
        } else {
            return match_sequence_strict(core, text);
        }
    }

    for i in 0..=text.len() {
        if match_sequence_strict(core, &text[i..]) {
            return true;
        }
    }

    false
}

fn match_sequence_strict<'a>(seq: &'a [RegexAst], mut text: &'a str) -> bool {
    for node in seq {
        match match_one(node, text) {
            Some(rest) => text = rest,
            None => return false,
        }
    }
    text.is_empty()
}

fn match_one<'a>(ast: &'a RegexAst, text: &'a str) -> Option<&'a str> {
    match ast {
        RegexAst::Literal(c) => {
            let mut chars = text.chars();
            let t = chars.next()?;
            if *c == t {
                Some(&text[t.len_utf8()..])
            } else {
                None
            }
        }
        RegexAst::AnyChar => {
            let mut chars = text.chars();
            let c = chars.next()?;
            Some(&text[c.len_utf8()..])
        }
        RegexAst::StartAnchor => Some(text), // handled in sequence
        RegexAst::EndAnchor => if text.is_empty() { Some("") } else { None },
        RegexAst::ZeroOrMore(inner) => {
            let mut t = text;
            while let Some(rest) = match_one(inner, t) {
                if rest == t {
                    break;
                }
                t = rest;
            }
            Some(t)
        }
        RegexAst::OneOrMore(inner) => {
            let mut t = match_one(inner, text)?;
            while let Some(rest) = match_one(inner, t) {
                if rest == t {
                    break;
                }
                t = rest;
            }
            Some(t)
        }
        RegexAst::ZeroOrOne(inner) => {
            if let Some(rest) = match_one(inner, text) {
                Some(rest)
            } else {
                Some(text)
            }
        }
        RegexAst::Group(inner) => match_one(inner, text),
        RegexAst::CharClass(set) => {
            let mut chars = text.chars();
            let c = chars.next()?;
            if char_class_match(c, set) {
                Some(&text[c.len_utf8()..])
            } else {
                None
            }
        }
        RegexAst::NegatedCharClass(set) => {
            let mut chars = text.chars();
            let c = chars.next()?;
            if !char_class_match(c, set) {
                Some(&text[c.len_utf8()..])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn char_class_match(c: char, set: &[CharSetItem]) -> bool {
    for item in set {
        match item {
            CharSetItem::Single(ch) => if *ch == c { return true; },
            CharSetItem::Range(start, end) => if *start <= c && c <= *end { return true; },
        }
    }
    false
}

fn eval_double_bracket_condition(cond: &str, context: &ShellContext) -> bool {
    let parts: Vec<&str> = cond.splitn(2, "=~").map(str::trim).collect();
    if parts.len() != 2 {
        return false;
    }

    let var_name = parts[0].trim_start_matches('$');
    let pattern = parts[1];

    let var_value = context.get_var(var_name).unwrap_or_default();

    regex_match(pattern, &var_value)
}

pub struct ShellContext {
    vars: std::collections::HashMap<String, String>,
}

impl ShellContext {
    pub fn get_var(&self, name: &str) -> Option<String> {
        self.vars.get(name).cloned()
    }
}
