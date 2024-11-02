# Rusty Bash (a.k.a. sushi 🍣 shell)


[![ubuntu-latest](https://github.com/shellgei/rusty_bash/actions/workflows/ubuntu.yml/badge.svg)](https://github.com/shellgei/rusty_bash/actions/workflows/ubuntu.yml)
[![macos-latest](https://github.com/shellgei/rusty_bash/actions/workflows/macos.yml/badge.svg)](https://github.com/shellgei/rusty_bash/actions/workflows/macos.yml)
![](https://img.shields.io/github/license/shellgei/rusty_bash)

[![demo](https://github.com/shellgei/rusty_bash/assets/1232918/13f9ae0b-45b6-4b89-ab5d-b523aadd09bf)](https://www.youtube.com/watch?v=RL8M6PZfDYo)

**IMPORTANT: the main branch is switched to the shell develped for articles on [SoftwareDesign](https://gihyo.jp/magazine/SD).**
（今までのメインブランチは、連載のものに比べて散らかりすぎなので、連載のものをmainに切り替えました。）

* [old main branch](https://github.com/shellgei/rusty_bash/tree/old_main)


## What's this?

A clone of Bash, which is developed as a hobby of our group and for monthly articles on SoftwareDesign magazine published by Gijutsu-Hyohron Co., Ltd.

## Quick Start

```bash
$ git clone https://github.com/shellgei/rusty_bash.git
$ cd rusty_bash
$ cargo run
・・・
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/sush`
ueda@uedaP1g6:main🌵~/GIT/rusty_bash(debug)🍣
```

## Install 

```bash
$ git clone https://github.com/shellgei/rusty_bash.git
$ cd rusty_bash
$ cargo build --release
### ↓  Change /bin/ to /usr/local/bin/ or another path in $PATH if you are using Mac or BSD ###
$ sudo cp target/release/sush /bin/
$ cp .sushrc_for_linux ~/.sushrc # edit if some errors occur
$ sush
ueda@uedaP1g6:main🌵~/GIT/rusty_bash🍣
```

## For Contributors 

Please give us issues or pull requests in a way you think sensible. We do not have a rigid rule at this stage. 

## List of Features

* :heavy_check_mark: :available
* :construction: :partially available (or having known bugs) 
* :no_good: : not implemented

### simple commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| command | :heavy_check_mark: | substitutions | :heavy_check_mark: | function definition | :heavy_check_mark: |


### compound commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| if | :heavy_check_mark: | while | :heavy_check_mark: | () | :heavy_check_mark: |
| {} | :heavy_check_mark: | case | :heavy_check_mark: | until | :no_good: | select | :no_good: |
| for | :heavy_check_mark: | [[ ]] | :heavy_check_mark: |

### control operator

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| \|\| | :heavy_check_mark: | && | :heavy_check_mark: | ; | :heavy_check_mark: |
| ;; | :heavy_check_mark: | \| | :heavy_check_mark: | & | :heavy_check_mark: |
| \|& | :heavy_check_mark: |

### expansion

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| brace `{a,b}` | :heavy_check_mark: | brace | :heavy_check_mark: | tilde | :heavy_check_mark: |
| arithmetic | :heavy_check_mark: | word splitting | :heavy_check_mark: | path name | :heavy_check_mark: |
| command substitution | :heavy_check_mark: | parameter/variable `$A ${A}` | :heavy_check_mark: | `${name:offset}, ${name:offset:length}` | :heavy_check_mark: |

### special parameters

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| $ | :heavy_check_mark: | ? | :heavy_check_mark: | * | :heavy_check_mark: |
| @ | :heavy_check_mark: | # | :heavy_check_mark: | - | :heavy_check_mark: |
| ! | :no_good: | _ | :heavy_check_mark: |

### builtin commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| cd | :heavy_check_mark: | pwd | :heavy_check_mark: | read | :construction: |
| exit | :heavy_check_mark: | source | :heavy_check_mark: | set | :construction: |
| shopt | :construction: | : | :heavy_check_mark: | . | :heavy_check_mark: | [ | :no_good: |
| alias | :heavy_check_mark: | bg | :construction: | bind | :no_good: |
| break | :heavy_check_mark: | builtin | :no_good: | caller | :no_good: |
| command | :no_good: | compgen | :construction: | complete | :construction: |
| compopt | :no_good: | continue | :no_good: | declare | :no_good: |
| dirs | :no_good: | disown | :no_good: | echo | :no_good: |
| enable | :no_good: | eval | :heavy_check_mark: | exec | :no_good: |
| fc | :no_good: | fg | :construction: | getopts | :no_good: |
| hash | :no_good: | help | :no_good: | history | :construction: |
| jobs | :construction: | kill | :no_good: | let | :no_good: |
| local | :heavy_check_mark: | logout | :no_good: | mapfile | :no_good: |
| popd | :no_good: | printf | :no_good: | pushd | :no_good: |
| read | :no_good: | readonly | :no_good: | return | :heavy_check_mark: |
| shift | :no_good: | suspend | :no_good: | test | :no_good: |
| times | :no_good: | trap | :no_good: | true | :heavy_check_mark: |
| type | :no_good: | typeset | :no_good: | ulimit | :no_good: |
| umask | :no_good: | unalias | :no_good: | unset | :construction: |
| wait | :construction: | export | :no_good: | false | :heavy_check_mark: |

### options

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| -c | :no_good: | -i | :heavy_check_mark: | -l, --login | :no_good: |
| -r | :no_good: | -s | :no_good: | -D | :no_good: |
| [-+]O | :no_good: | -- | :no_good: | --debugger | :no_good: |
| --dimp-po-strings | :no_good: | --help | :no_good: | --init-file | :no_good: |
| --rcfile | :no_good: | --noediting | :no_good: | --noprofile | :no_good: |
| --norc | :no_good: | --posix | :no_good: | --restricted | :no_good: |
| -v, --verbose | :no_good: | --version | :heavy_check_mark: | -e | :heavy_check_mark: |
| --pipefail | :heavy_check_mark: |  |  |  |  |


### shopt 

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| autocd | :no_good: | cdable_vars | :no_good: | cdspell | :no_good: |
| checkhash | :no_good: | checkjobs | :no_good: | checkwinsize | :no_good: |
| cmdhist | :no_good: | compat31 | :no_good: | compat32 | :no_good: |
| compat40 | :no_good: | compat41 | :no_good: | dirspell | :no_good: |
| dotglob | :no_good: | execfail | :no_good: | expand_aliases | :no_good: |
| extdebug | :no_good: | extglob | :heavy_check_mark: | extquote | :no_good: |
| failglob | :no_good: | force_fignore | :no_good: | globstar | :no_good: |
| gnu_errfmt | :no_good: | histappend | :no_good: | histreedit | :no_good: |
| histverify | :no_good: | hostcomplete | :no_good: | huponexit | :no_good: |
| interactive_comments | :no_good: | lastpipe | :no_good: | lithist | :no_good: |
| login_shell | :no_good: | mailwarn | :no_good: | no_empty_cmd_completion | :no_good: |
| nocaseglob | :no_good: | nocasematch | :no_good: | nullglob | :no_good: |
| progcomp | :no_good: | promptvars | :no_good: | restricted_shell | :no_good: |
| shift_verbose | :no_good: | sourcepath | :no_good: | xpg_echo | :no_good: |

### variables

Born Shell Variables

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| CDPATH | :no_good: | HOME | :heavy_check_mark: | IFS | :no_good: |
| MAIL | :no_good: | MAILPATH | :no_good: | OPTARG | :no_good: |
| OPTIND | :no_good: | PATH | :heavy_check_mark: | PS1 | :heavy_check_mark: |
| PS2 | :heavy_check_mark: | | | | |

Bash Variables

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| _ | :heavy_check_mark: | BASH | :no_good: | BASHOPTS | :no_good: |
| BASHPID | :heavy_check_mark: | BASH_ALIASES | :no_good: | BASH_ARGC | :no_good: |
| BASH_ARGV | :no_good: | BASH_ARGV0 | :no_good: | BASH_CMDS | :no_good: |
| BASH_COMMAND | :no_good: | BASH_COMPAT | :no_good: | BASH_ENV | :no_good: |
| BASH_EXECUTION_STRING | :no_good: | BASH_LINENO | :no_good: | BASH_LOADABLES_PATH | :no_good: |
| BASH_REMATCH | :no_good: | BASH_SOURCE | :no_good: | BASH_SUBSHELL | :heavy_check_mark: |
| BASH_VERSINFO | :heavy_check_mark: | BASH_VERSION | :heavy_check_mark: | BASH_XTRACEFD | :no_good: |
| CHILD_MAX | :no_good: | COLUMNS | :no_good: | COMP_CWORD | :no_good: |
| COMP_LINE | :no_good: | COMP_POINT | :no_good: | COMP_TYPE | :no_good: |
| COMP_KEY | :no_good: | COMP_WORDBREAKS | :no_good: | COMP_WORDS | :no_good: |
| COMPREPLY | :no_good: | COPROC | :no_good: | DIRSTACK | :no_good: |
| EMACS | :no_good: | ENV | :no_good: | EPOCHREALTIME | :no_good: |
| EPOCHSECONDS | :no_good: | EUID | :no_good: | EXECIGNORE | :no_good: |
| FCEDIT | :no_good: | FIGNORE | :no_good: | FUNCNAME | :no_good: |
| FUNCNEST | :no_good: | GLOBIGNORE | :no_good: | GROUPS | :no_good: |
| histchars | :no_good: | HISTCMD | :no_good: | HISTCONTROL | :no_good: |
| HISTFILE | :heavy_check_mark: | HISTFILESIZE | :heavy_check_mark: | HISTIGNORE | :no_good: |
| HISTSIZE | :no_good: | HISTTIMEFORMAT | :no_good: | HOSTFILE | :no_good: |
| HOSTNAME | :no_good: | HOSTTYPE | :heavy_check_mark: | IGNOREEOF | :no_good: |
| INPUTRC | :no_good: | INSIDE_EMACS | :no_good: | LANG | :heavy_check_mark: |
| LC_ALL | :no_good: | LC_COLLATE | :no_good: | LC_CTYPE | :no_good: |
| LC_MESSAGES | :no_good: | LC_NUMERIC | :no_good: | LC_TIME | :no_good: |
| LINENO | :heavy_check_mark: | LINES | :no_good: | MACHTYPE | :heavy_check_mark: |
| MAILCHECK | :no_good: | MAPFILE | :no_good: | OLDPWD | :heavy_check_mark: |
| OPTERR | :no_good: | OSTYPE | :heavy_check_mark: | PIPESTATUS | :heavy_check_mark: |
| POSIXLY_CORRECT | :no_good: | PPID | :no_good: | PROMPT_COMMAND | :no_good: |
| PROMPT_DIRTRIM | :no_good: | PS0 | :no_good: | PS3 | :no_good: |
| PS4 | :heavy_check_mark: | PWD | :heavy_check_mark: | RANDOM | :no_good: |
| READLINE_ARGUMENT | :no_good: | READLINE_LINE | :no_good: | READLINE_MARK | :no_good: |
| READLINE_POINT | :no_good: | REPLY | :no_good: | SECONDS | :no_good: |
| SHELL | :heavy_check_mark: | SHELLOPTS | :no_good: | SHLVL | :heavy_check_mark: |
| SRANDOM | :no_good: | TIMEFORMAT | :no_good: | TMOUT | :no_good: |
| TMPDIR | :no_good: | UID | :no_good: | | |

### beyond Bash

|features | status |
|-------------------|----|
| branch display in prompt | :heavy_check_mark: |

## Thanks to

Partially in Japanese.

* blog articles
    * [Rustでシェル作った | κeenのHappy Hacκing Blog](https://keens.github.io/blog/2016/09/04/rustdeshierutsukutta/)
    * [Rustで始める自作シェル その1 | ぶていのログでぶログ](https://tech.buty4649.net/entry/2021/12/19/235124)
    * [Rustのターミナル操作crateいろいろ | meganehouser](https://meganehouser.github.io/2019-12-11_rust-terminal-crates.html)
    * [原理原則で理解するフォアグラウンドプロセスとバックグラウンドプロセスの違い | @tajima_taso](https://qiita.com/tajima_taso/items/c5553762af5e1a599fed)
    * [Bashタブ補完自作入門 | Cybouzu Inside Out](https://blog.cybozu.io/entry/2016/09/26/080000)



## Copyright

© 2022-2024 shellgei group

- Ryuichi Ueda: [@ry@mi.shellgei.org](https://mi.shellgei.org/@ru), [@ryuichiueda@misskey.io](https://misskey.io/@ryuichiueda)
- [@caro@mi.shellgei.org](https://mi.shellgei.org/@caro)
