//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

pub fn split(sw: &str, ifs: &str, prev_char: Option<char>) -> Vec<(String, bool)> {
    //bool: true if it should remain
    if ifs.is_empty() {
        return vec![(sw.to_string(), false)];
    }

    if ifs.chars().all(|c| " \t\n".contains(c)) {
        split_str_normal(sw, ifs)
    } else {
        split_str_special(sw, ifs, prev_char)
    }
}

fn scanner_blank(s: &str, blank: &[char]) -> usize {
    let mut ans = 0;
    let mut esc = false;

    for ch in s.chars() {
        if esc || ch == '\\' {
            esc = !esc;
            ans += ch.len_utf8();
            continue;
        }

        if blank.contains(&ch) {
            ans += ch.len_utf8();
        } else {
            break;
        }
    }

    ans
}

fn scanner_ifs_blank(s: &str, blank: &[char], delim: &[char]) -> usize {
    let mut ans = 0;
    let mut esc = false;

    for ch in s.chars() {
        if esc || ch == '\\' {
            esc = !esc;
            ans += ch.len_utf8();
            continue;
        }

        if delim.contains(&ch) {
            ans += ch.len_utf8();
            ans += scanner_blank(&s[ans..], blank);
            return ans;
        } else if blank.contains(&ch) {
            ans += ch.len_utf8();
        } else {
            break;
        }
    }

    ans
}

fn split_str_special(s: &str, ifs: &str, prev_char: Option<char>) -> Vec<(String, bool)> {
    let mut ans = vec![];
    let mut remaining = s.to_string();

    let shave_prev = match prev_char {
        None => true,
        Some(c) => " \t\n".contains(c),
    };

    let blank: Vec<char> = ifs.chars().filter(|s| " \t\n".contains(*s)).collect();
    let delim: Vec<char> = ifs.chars().filter(|s| !" \t\n".contains(*s)).collect();

    if shave_prev {
        let len = scanner_blank(&remaining, &blank);
        let tail = remaining.split_off(len);
        remaining = tail;
    }

    while !remaining.is_empty() {
        let len = scanner_word(&remaining, ifs);
        let tail = remaining.split_off(len);

        ans.push((remaining.to_string(), true));
        remaining = tail;

        let len = scanner_ifs_blank(&remaining, &blank, &delim);
        if len > 0 {
            remaining = remaining.split_off(len);
            if remaining.is_empty() {
                ans.push(("".to_string(), false));
            }
        }
    }

    if ans.is_empty() {
        ans.push(("".to_string(), false));
        ans.push(("".to_string(), false));
    }
    ans
}

fn split_str_normal(s: &str, ifs: &str) -> Vec<(String, bool)> {
    let mut esc = false;
    let mut from = 0;
    let mut pos = 0;
    let mut ans = vec![];

    for c in s.chars() {
        pos += c.len_utf8();
        if esc || c == '\\' {
            esc = !esc;
            continue;
        }

        if ifs.contains(c) {
            let sw = s[from..pos - c.len_utf8()].to_string();
            ans.push((sw, false));
            from = pos;
        }
    }

    ans.push((s[from..].to_string(), false));

    ans
}

fn scanner_word(s: &str, ifs: &str) -> usize {
    let mut ans = 0;
    let mut esc = false;

    for ch in s.chars() {
        if esc || ch == '\\' {
            esc = !esc;
            ans += ch.len_utf8();
            continue;
        }

        if ifs.contains(ch) {
            return ans;
        }

        ans += ch.len_utf8();
    }

    ans
}
