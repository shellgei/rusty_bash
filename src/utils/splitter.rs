//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-FileCopyrightText: 2026 @ru@mi.shiellgei.org
//SPDX-License-Identifier: BSD-3-Clause

pub fn split(sw: &str, ifs: &str, strip_left: bool)
-> Option<Vec<(String, bool)>> {
    //bool: true if it should remain
    if ifs.is_empty() {
        //return vec![(sw.to_string(), false)];
        return None;
    }

    if ifs.chars().all(|c| " \t\n".contains(c)) {
        split_str_normal(sw, ifs)
    } else {
        split_str_custom_ifs(&mut sw.to_string(), ifs, strip_left)
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

fn eat_word(remaining: &mut String, ans: &mut Vec<(String, bool)>, ifs: &str) {
    let len = scanner_word(&remaining, ifs);
    let tail = remaining.split_off(len);
    ans.push((remaining.to_string(), true));
    *remaining = tail;
}

fn shave_blank(remaining: &mut String, ans: &mut Vec<(String, bool)>,
               blank: &Vec<char>, delim: &Vec<char>) {
    let len = scanner_ifs_blank(&remaining, &blank, &delim);
    if len > 0 {
        *remaining = remaining.split_off(len);
        if remaining.is_empty() {
            ans.push(("".to_string(), false));
        }
    }
}

fn split_str_custom_ifs(remaining: &mut String, ifs: &str, strip_left: bool)
-> Option<Vec<(String, bool)>> {
    let mut ans = vec![];
    let mut shaved = false;

    let blank: Vec<char> = ifs.chars().filter(|s| " \t\n".contains(*s)).collect();
    let delim: Vec<char> = ifs.chars().filter(|s| !" \t\n".contains(*s)).collect();

    if strip_left {
        let len = scanner_blank(&remaining, &blank);
        shaved = len > 0;
        *remaining = remaining.split_off(len);
    }

    while !remaining.is_empty() {
        eat_word(remaining, &mut ans, ifs);
        shave_blank(remaining, &mut ans, &blank, &delim);
    }

    if ans.is_empty() {
        ans.push(("".to_string(), false));
    }

    if shaved && ans.len() < 2 {
        //if the string is modified, the splitting is applied.
        ans.push(("".to_string(), false));
    }

    Some(ans)
}

fn split_str_normal(s: &str, ifs: &str)
-> Option<Vec<(String, bool)>> {
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

    Some(ans)
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
