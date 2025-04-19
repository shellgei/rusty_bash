//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2025 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

//mod completion;
//mod key;

use std::env;
use crate::utils::file_check;
use crate::ShellCore;
use crate::utils::file;
use crate::error::input::InputError;
use std::io::{BufReader, BufRead};
use std::fs;
use std::fs::File;
use std::sync::atomic::Ordering::Relaxed;
use std::path::Path;
use nix::unistd;
use nix::unistd::User;
use rustyline::{Context, Helper, Editor, Config, EditMode, CompletionType};
use rustyline::validate::{MatchingBracketValidator, Validator, ValidationContext, ValidationResult};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter, CmdKind};
use rustyline::hint::Hinter;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::io::{stdout, Write};

pub struct SushHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    colored_prompt: String,
}

impl Helper for SushHelper {}

// コマンド候補取得（仮）
fn get_commands(prefix: &str) -> Vec<Pair> {
    // PATHから
    let commands_set: HashSet<String> = env::var("PATH")
        .ok()
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            if meta.is_file() && (meta.permissions().mode() & 0o111 != 0) {
                entry.file_name().into_string().ok().filter(|name| name.starts_with(prefix))
            } else {
                None
            }
        })
        .collect();

    // 仮設なのでbuilt-inとかalias無い・・・

    // 重複を除いたPairに変換しソート
    let mut pairs: Vec<Pair> = commands_set
        .into_iter()
        .map(|name| Pair {
            display: name.clone(),
            replacement: name,
        })
        .collect();
    pairs.sort_by(|a, b| a.display.cmp(&b.display));
    pairs
}

impl Completer for SushHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let text = &line[..pos];
        let tokens: Vec<&str> = text.split_whitespace().collect();
        // 最初のトークン入力中かつ '/' が含まれていなければコマンド補完
        if tokens.is_empty() || (tokens.len() == 1 && !tokens[0].contains('/')) {
            let prefix = if tokens.is_empty() { "" } else { tokens[0] };
            let completions = get_commands(prefix);
            let start = text.find(prefix).unwrap_or(0);
            Ok((start, completions))
        } else {
            // それ以外はファイル補完
            self.completer.complete(line, pos, ctx)
        }
    }
}

impl Hinter for SushHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

// よくわからないのでサンプルそのまま
impl Highlighter for SushHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

// よくわかんないけど、シェル芸で便利そう！！！
impl Validator for SushHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input().to_string();
        // bracket matching
        let result = self.validator.validate(ctx)?;
        if let ValidationResult::Incomplete = result {
            return Ok(ValidationResult::Incomplete);
        }
        // unbalanced quotes
        if unbalanced_quotes(&input) {
            return Ok(ValidationResult::Incomplete);
        }
        Ok(ValidationResult::Valid(None))
    }
}

fn unbalanced_quotes(s: &str) -> bool {
    let mut single = 0;
    let mut double = 0;
    let mut escaped = false;
    for c in s.chars() {
        if c == '\\' && !escaped {
            escaped = true;
            continue;
        }
        if c == '\'' && !escaped {
            single ^= 1;
        }
        if c == '"' && !escaped {
            double ^= 1;
        }
        escaped = false;
    }
    single != 0 || double != 0
}

fn oct_string(s: &str) -> bool {
    if s.chars().nth(0) != Some('\\') {
        return false;
    }

    for i in 1..4 {
        match s.chars().nth(i) {
            Some(c) => {
                if c < '0' || '9' < c {
                    return false;
                }
            },
            _ => return false,
        }
    }

    true
}

fn oct_to_hex_in_str(from: &str) -> String {
    let mut i = 0;
    let mut pos = vec![];

    for ch in from.chars() {
        if oct_string(&from[i..]) {
            pos.push(i);
        }
        i += ch.len_utf8();
    }

    let mut prev = 0;
    let mut ans = String::new();
    for p in pos {
        ans += &from[prev..p];
        if let Ok(n) = u32::from_str_radix(&from[p+1..p+4], 8) {
            ans += &char::from_u32(n).unwrap().to_string();
        }
        prev = p+4;
    }
    ans += &from[prev..];
    ans
}

fn get_branch(cwd: &String) -> String {
    let mut dirs: Vec<String> = cwd.split("/").map(|s| s.to_string()).collect();
    while dirs.len() > 0 {
        let path = dirs.join("/") + "/.git/HEAD";
        dirs.pop();

        if ! file_check::is_regular_file(&path) {
            continue;
        }

        if let Ok(f) = File::open(Path::new(&path)) {
            let r = BufReader::new(f);
            return match r.lines().next() {
                Some(Ok(l)) => l.replace("ref: refs/heads/", "") + "🌵",
                _ => "".to_string(),
            };
        }
    }

    "".to_string()
}

fn make_prompt_string(raw: &str) -> String {
    let uid = unistd::getuid();
    let user = match User::from_uid(uid) {
        Ok(Some(u)) => u.name,
        _ => "".to_string(),
    };
    let hostname = match unistd::gethostname() {
        Ok(h) => file::oss_to_name(&h),
        _ => "".to_string(),
    };

    let homedir = match User::from_uid(uid) {
        Ok(Some(u)) => file::buf_to_name(&u.dir),
        _ => "".to_string(),
    };
    let mut cwd = match unistd::getcwd() {
        Ok(p) => file::buf_to_name(&p),
        _ => "".to_string(),
    };
    let branch = get_branch(&cwd);

    if cwd.starts_with(&homedir) {
        cwd = cwd.replacen(&homedir, "~", 1);
    }

    raw.replace("\\u", &user)
       .replace("\\h", &hostname)
       .replace("\\w", &cwd)
       .replace("\\b", &branch)
       .to_string()
}

fn parse_visible_prompt(prompt: &str) -> (String, String) {
    let mut display = String::new();
    let mut hidden = String::new();
    let mut chars = prompt.chars().peekable();

    while let Some(c) = chars.next() {
        // 非表示ブロック開始"\["を検出
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next(); // "\["
                    let mut block = String::new();
                    // "\]" が来るまでブロック内容を収集
                    while let Some(ch) = chars.next() {
                        if ch == '\\' {
                            if let Some(&maybe_end) = chars.peek() {
                                if maybe_end == ']' {
                                    chars.next(); // "\]"
                                    break;
                                }
                            }
                        }
                        block.push(ch);
                    }
                    // hiddenに追加
                    hidden.push_str(&block);
                    // ブロックがウィンドウタイトル用（ESCの後に']'で始まる）はdisplayに追加しない
                    if !block.starts_with("\u{1b}]") {
                        display.push_str(&block);
                    }
                    continue;
                }
            }
        }
        // 非表示ブロック外の文字はdisplay
        display.push(c);
    }
    (display, hidden)
}

pub struct Terminal {
    rl: Editor<SushHelper, DefaultHistory>,
}

impl Terminal {
    pub fn new(core: &mut ShellCore) -> Terminal {
        let config = Config::builder()
            .edit_mode(EditMode::Emacs)
            .auto_add_history(true)
            .color_mode(rustyline::ColorMode::Enabled)
            .completion_type(CompletionType::List)
            .build();
        let mut rl: Editor<SushHelper, DefaultHistory> = Editor::with_config(config).unwrap();
        let helper = SushHelper { completer: FilenameCompleter::new(), highlighter: MatchingBracketHighlighter::new(), validator: MatchingBracketValidator::new(), colored_prompt: String::new() };
        rl.set_helper(Some(helper));
        if let Ok(history_file) = core.db.get_param("HISTFILE") {
            if !history_file.is_empty() {
                let path = std::path::PathBuf::from(&history_file);
                let _ = rl.load_history(&path);
            }
        }
        for entry in core.history.iter().rev() {
            let _ = rl.add_history_entry(entry);
        }
        Terminal { rl }
    }

    pub fn read_line(&mut self, core: &mut ShellCore, prompt: &str) -> Result<String, InputError> {
        let raw = core.db.get_param(prompt).unwrap_or_default();
        let replaced = make_prompt_string(&raw);
        let ansi = oct_to_hex_in_str(&replaced);
        let (display, hidden) = parse_visible_prompt(&ansi);
        // output hidden parts for ANSI sequences
        print!("{}", hidden);
        stdout().flush().unwrap();
        if let Some(helper) = self.rl.helper_mut() {
            helper.colored_prompt = display.clone();
        }
        match self.rl.readline(&display) {
            Ok(line) => Ok(line),
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C, print newline
                println!();
                core.sigint.store(true, Relaxed);
                Err(InputError::Interrupt)
            },
            Err(ReadlineError::Eof) => Err(InputError::Eof),
            Err(_) => Err(InputError::Interrupt),
        }
    }

    pub fn save_history(&mut self, path: &std::path::Path) {
        let _ = self.rl.save_history(path);
    }
}
