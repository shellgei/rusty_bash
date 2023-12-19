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
