#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

res=$($com <<< 'cd /; pwd')
[ "$res" = "/" ] || err $LINENO

res=$($com <<< 'rm -f /tmp/link; cd /tmp; mkdir -p hoge; ln -s hoge link; cd link; pwd -L; pwd -P')
[ "$res" = "/tmp/link
/tmp/hoge" ] ||
[ "$res" = "/tmp/link
/private/tmp/hoge" ] || err $LINENO

res=$($com <<< 'pwd -a 2>/tmp/rusty_bash; cat /tmp/rusty_bash')
[ "$res" = "sush: pwd: -a: invalid option
pwd: usage: pwd [-LP]" ] || err $LINENO

echo aaaaaaaaaaaaaaaa > /tmp/hoge.txt
res=$($com <<< 'source /tmp/hoge.txt')
[ "$?" = "127" ] || err $LINENO

echo '(' > /tmp/hoge.txt
res=$($com <<< 'source /tmp/hoge.txt')
[ "$?" = "2" ] || err $LINENO

res=$($com <<< 'compgen -W "aaa abc aac" -- aa')
[ "$res" = "aaa
aac" ] || err $LINENO

b=$(compgen -f / | wc -l )
res=$($com <<< 'compgen -f / | wc -l')
[ "$res" = "$b" ] || err $LINENO

b=$(compgen -d /etc | wc -l )
res=$($com <<< 'compgen -d /etc | wc -l')
[ "$res" = "$b" ] || err $LINENO

b=$(compgen -d -- /etc | wc -l )
res=$($com <<< 'compgen -d -- /etc | wc -l')
[ "$res" = "$b" ] || err $LINENO

b=$(cd ; compgen -f . | wc -l )
res=$($com <<< 'cd ; compgen -f . | wc -l')
[ "$res" = "$b" ] || err $LINENO

res=$($com <<< 'eval "echo a" b')
[ "$res" = "a b" ] || err $LINENO

res=$($com <<< 'eval "(" echo abc ")" "|" rev')
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'A=aaa ; unset A ; echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=aaa ; unset -f A ; echo $A')
[ "$res" = "aaa" ] || err $LINENO

res=$($com <<< 'A=aaa ; unset -v A ; echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A () { echo aaa ; } ; unset -v A ; A')
[ "$res" = "aaa" ] || err $LINENO

res=$($com <<< 'A () { echo aaa ; } ; unset -f A ; A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A () { echo aaa ; } ; unset A ; A')
[ "$res" = "" ] || err $LINENO

# source command

res=$($com <<< 'echo $PS1')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'case aaa in aaa) return && echo NG ;; esac')
[ "$?" = "2" ] || err $LINENO
[ "$res" = "" ] || err $LINENO

# break command

$com <<< 'while true ; do break ; done'
#[ "$res" == "" ] || err $LINENO

res=$($com <<< 'while true ; do break ; echo NG ; done')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'while true ; do while true ; do break ; done ; echo OK ;break ; done; echo OK')
[ "$res" == "OK
OK" ] || err $LINENO

res=$($com <<< 'while true ; do while true ; do break 2 ; done ; echo NG ; done ; echo OK')
[ "$res" == "OK" ] || err $LINENO

res=$($com <<< 'while true ; do while true ; do break 10 ; done ; echo NG ; done ; echo OK')
[ "$res" == "OK" ] || err $LINENO

# read

res=$($com <<< 'seq 2 | while read a ; do echo $a ; done ; echo $a ; echo A')
[ "$res" == "1
2

A" ] || err $LINENO

res=$($com <<< 'A=BBB; seq 2 | while read $A ; do echo $BBB ; done')
[ "$res" == "1
2" ] || err $LINENO

res=$($com <<< 'echo あ い う | while read a b ; do echo $a ; echo $b ; done')
[ "$res" == "あ
い う" ] || err $LINENO

# set command

res=$($com <<< 'set -- a b c ; echo $2')
[ "$res" == "b" ] || err $LINENO

# shopt command

res=$($com <<< 'shopt -u extglob ; echo @(a)')
[ "$res" == "@(a)" ] || err $LINENO

res=$($com <<< 'shopt -u extglob
echo @(a)')
[ "$?" == "2" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

echo $0 >> ./ok

