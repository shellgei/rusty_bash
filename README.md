# Rusty Bash (a.k.a. sushi 🍣 shell)


**IMPORTANT: I switche the main branch to the shell develped for articles on [SoftwareDesign](https://gihyo.jp/magazine/SD).**

* [old main branch](https://github.com/shellgei/rusty_bash/tree/alpha_main)

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

## list of features

* :heavy_check_mark: :available
* :construction: :partially available (or having known bugs) 
* :no_good: : not implemented


### compound commands

|features | status |features | status |features | status |
|-------------------|----|-------------------|----|-------------------|----|
| if | :no_good: | while | :heavy_check_mark: | () | :heavy_check_mark: | 
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
| shopt | :no_good: | : | :no_good: | . | :no_good: | [ | :no_good: |
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
| times | :no_good: | trap | :no_good: | true | :no_good: |
| type | :no_good: | typeset | :no_good: | ulimit | :no_good: |
| umask | :no_good: | unalias | :no_good: | unset | :no_good: |
| wait | :no_good: | export | :no_good: | false | :no_good: |

