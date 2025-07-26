//SPDX-FileCopyrightText: 2025 Hugo Fortin
//SPDX-License-Identifier: BSD-3-Clause

///// Custom and ultra-light Regex implementation /////

pub fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' => regex.push_str(r"\."),
            '[' => {
                regex.push('[');
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '!' || next_ch == '^' {
                        regex.push('^');
                        chars.next();
                    }
                }
                while let Some(c) = chars.next() {
                    regex.push(c);
                    if c == ']' {
                        break;
                    }
                }
            }
            '\\' => regex.push_str(r"\\"),
            c if is_metachar(c) => {
                regex.push('\\');
                regex.push(c);
            }
            c => regex.push(c),
        }
    }

    regex.push('$');
    regex
}

fn is_metachar(c: char) -> bool {
    matches!(c, '^' | '$' | '.' | '+' | '(' | ')' | '|' | '{' | '}' | '\\')
}

pub fn shell_pattern_to_regex(pattern: &str) -> String {
    glob_to_regex(pattern)
}

pub fn naive_glob_match(text: &str, regex: &str) -> bool {
    text == regex.trim_start_matches('^').trim_end_matches('$')
}
