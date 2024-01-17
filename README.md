# Rusty Bash (a.k.a. sushi 🍣 shell)

[![ubuntu-latest](https://github.com/shellgei/rusty_bash/actions/workflows/ubuntu.yml/badge.svg)](https://github.com/shellgei/rusty_bash/actions/workflows/ubuntu.yml)
[![macos-latest](https://github.com/shellgei/rusty_bash/actions/workflows/macos.yml/badge.svg)](https://github.com/shellgei/rusty_bash/actions/workflows/macos.yml)
![](https://img.shields.io/github/license/shellgei/rusty_bash)


**IMPORTANT: the main branch is switched to the shell develped for articles on [SoftwareDesign](https://gihyo.jp/magazine/SD).**
（今までのメインブランチは、連載のものに比べて散らかりすぎなので、連載のものをmainに切り替えました。）

* [old main branch](https://github.com/shellgei/rusty_bash/tree/old_main)

## What's this?

A clone of Bash, which is developed as a hobby of our group and for monthly articles on SoftwareDesign magazine published by Gijutsu-Hyohron Co., Ltd.

## Quick Start

```bash
$ git clone https://github.com/shellgei/rusty_bash.git
$ cd rusty_bash
$ cargo build
...
🍣 echo hello
hello
🍣 exit
```

## For Contributors 

Since the main branch must synchronize the articles, further developments are reflected in the following branches. We prepared dev-* branches for adding your codes with pull requests.
* [dev-builtins](https://github.com/shellgei/rusty_bash/tree/dev-builtins): for builtin commands 
* [dev-signal](https://github.com/shellgei/rusty_bash/tree/dev-signal): for development on signal
* [dev-compounds](https://github.com/shellgei/rusty_bash/tree/dev-compounds): for development on compoundss
* [dev-args](https://github.com/shellgei/rusty_bash/tree/dev-args): for development on arguments
* [dev-test](https://github.com/shellgei/rusty_bash/tree/dev-test): for development of a nice test system and test cases (we need more sophisticated test)
* [dev-others](https://github.com/shellgei/rusty_bash/tree/dev-others): for development of other features

These dev-* branches will be merged to the main branch depending on the situation of the articles. You can also propose some dev-foobar branches. 

## List of Features

* :heavy_check_mark: :available
* :construction: :partially available (or having known bugs) 
* :no_good: : not implemented


### compound commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| if | :heavy_check_mark: | while | :heavy_check_mark: | () | :heavy_check_mark: | 
| {} | :heavy_check_mark: | case | :no_good: | until | :no_good: | select | :no_good: | 
| for | :no_good: |


### control operator

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| \|\| | :heavy_check_mark: | && | :heavy_check_mark: | ; | :heavy_check_mark: |
| ;; | :heavy_check_mark: | \| | :heavy_check_mark: | & | :heavy_check_mark: |
| \|& | :heavy_check_mark: | 

### builtin commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| cd | :heavy_check_mark: | pwd | :heavy_check_mark: | read | :no_good: |
| exit | :heavy_check_mark: | source | :no_good: | set | :no_good: | 
| shopt | :no_good: | : | :heavy_check_mark: | . | :no_good: | [ | :no_good: |
| alias | :no_good: | bg | :no_good: | bind | :no_good: |
| break | :no_good: | builtin | :no_good: | caller | :no_good: |
| command | :no_good: | compgen | :no_good: | complete | :no_good: |
| compopt | :no_good: | continue | :no_good: | declare | :no_good: |
| dirs | :no_good: | disown | :no_good: | echo | :no_good: |
| enable | :no_good: | eval | :no_good: | exec | :no_good: |
| fc | :no_good: | fg | :no_good: | getopts | :no_good: |
| hash | :no_good: | help | :no_good: | history | :no_good: |
| jobs | :no_good: | kill | :no_good: | let | :no_good: |
| local | :no_good: | logout | :no_good: | mapfile | :no_good: |
| popd | :no_good: | printf | :no_good: | pushd | :no_good: |
| read | :no_good: | readonly | :no_good: | return | :no_good: |
| shift | :no_good: | suspend | :no_good: | test | :no_good: |
| times | :no_good: | trap | :no_good: | true | :heavy_check_mark: |
| type | :no_good: | typeset | :no_good: | ulimit | :no_good: |
| umask | :no_good: | unalias | :no_good: | unset | :no_good: |
| wait | :no_good: | export | :no_good: | false | :heavy_check_mark: |

## Thanks to

Partially in Japanese.

* blog articles
    * [Rustでシェル作った | κeenのHappy Hacκing Blog](https://keens.github.io/blog/2016/09/04/rustdeshierutsukutta/)
    * [Rustで始める自作シェル その1 | ぶていのログでぶログ](https://tech.buty4649.net/entry/2021/12/19/235124)
    * [Rustのターミナル操作crateいろいろ | meganehouser](https://meganehouser.github.io/2019-12-11_rust-terminal-crates.html)
    * [原理原則で理解するフォアグラウンドプロセスとバックグラウンドプロセスの違い | @tajima_taso](https://qiita.com/tajima_taso/items/c5553762af5e1a599fed)

