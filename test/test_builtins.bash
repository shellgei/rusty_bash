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

res=$($com <<< 'eval -- "A=(a b)"; echo ${A[@]}')
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

# builtin command
#
res=$($com <<< 'builtin cd; pwd')
[ "$res" = ~ ] || err $LINENO

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

# continue command

res=$($com <<< 'seq 2 | while read d ; do echo x; continue; echo NG ; done')
[ "$res" == "x
x" ] || err $LINENO

res=$($com <<< 'seq 2 | while read d ; do for a in a b ; do echo x; continue 2 ; done ; echo NG ; done')
[ "$res" == "x
x" ] || err $LINENO

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

# local

res=$($com -c 'A=1 ; f () { local -a A ; A[1]=123 ; echo ${A[@]} ; } ; f ; echo $A')
[[ "$res" == '123
1' ]] || err $LINENO

res=$($com -c 'A=1 ; f () { local -a A ; A=(2 123) ; echo ${A[@]} ; } ; f ; echo $A')
[[ "$res" == '2 123
1' ]] || err $LINENO

res=$($com -c 'A=1 ; f () { local -A A ; A[aaa]=bbb ; echo ${A[@]} ; } ; f ; echo $A')
[[ "$res" == 'bbb
1' ]] || err $LINENO

res=$($com -c 'A=1 ; f () { local -a A=(2 123) ; echo ${A[@]} ; } ; f ; echo $A')
[[ "$res" == '2 123
1' ]] || err $LINENO

res=$($com -c 'A=1 ; f () { local A=5 ; A=4 ; } ; f ; echo $A')
[[ "$res" == '1' ]] || err $LINENO

res=$($com <<< 'f() { local a=1 ; local "a" && echo "$a" ; } ; f')
[ "$res" = "1" ] || err $LINENO

### declare ###

res=$($com -c 'A=1 ; f () { local A ; declare -r A ; A=123 ; } ; f')
[[ "$?" -eq 1 ]] || err $LINENO

res=$($com -c 'f () { local A ; declare -r A ; A=123 ; } ; f; A=3 ; echo $A')
[[ "$res" -eq 3 ]] || err $LINENO

res=$($com -c 'A=1 ; declare -r A ; f () { local A ; A=123 ; } ; f')
[[ "$?" -eq 1 ]] || err $LINENO

res=$($com -c 'A=1 ; declare -r A ; A=(3 4)')
[[ "$?" -eq 1 ]] || err $LINENO

### command ###

res=$($com -c 'command cd /; pwd')
[[ "$res" == / ]] || err $LINENO

res=$($com -c 'command cd /; pwd')
[[ "$res" == / ]] || err $LINENO

### getopts ###

res=$($com -c '
getopts xyz opt -x -y
echo $opt
getopts xyz opt -x -y
echo $opt
')

[[ "$res" == "x
y" ]] || err $LINENO

res=$($com -c '
getopts x:y:z opt -x hoge -y fuge -z
echo $opt $OPTARG
getopts x:y:z opt -x hoge -y fuge -z
echo $opt $OPTARG
getopts x:y:z opt -x hoge -y fuge -z
echo $opt $OPTARG
')

[[ "$res" == 'x hoge
y fuge
z' ]] || err $LINENO


res=$($com -c '
getopts n: opt -n aho boke fuge
getopts n: opt -n aho boke fuge
getopts n: opt -n aho boke fuge
echo $?
echo $OPTIND
')
[[ "$res" == '1
3' ]] || err $LINENO

res=$($com <<< 'set -- -s --; echo $@
getopts s flag "$@"; res=$?
echo flag:$flag OPTIND:$OPTIND exit:$res
getopts s flag "$@"; res=$?
echo flag:$flag OPTIND:$OPTIND exit:$res
')
[ "$res" = "-s --
flag:s OPTIND:2 exit:0
flag:? OPTIND:3 exit:1" ] || err $LINENO

### printf ###

res=$($com <<< 'printf -v a %s bbb &> /dev/null; echo $a')
[ "$res" = "bbb" ] || err $LINENO

res=$($com <<< 'printf -v a %s &> /dev/null; echo $a')
[ "$?" -eq 0 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'printf -v a bb cc dd &> /dev/null; echo $a')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'printf -v a[3] bb cc dd &> /dev/null; echo ${a[@]}')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'printf -v a[3] bb cc dd &> /dev/null; echo ${a[3]}')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'printf %s abc > /dev/null')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'printf %s abc &> /dev/null')
[ "$res" = "" ] || err $LINENO

### trap ###
#
res=$($com <<< 'trap "echo hoge" 4') # 4 (SIGILL) is forbidden by signal_hook
[ $? -eq 1 ] || err $LINENO

res=$($com <<< 'trap "echo hoge" QUIT; kill -3 $$; sleep 1')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'trap "echo hoge" 444444') 
[ $? -eq 1 ] || err $LINENO

res=$($com <<< 'trap "echo hoge" EXIT; echo fuge') 
[ "$res" = "fuge
hoge" ] || err $LINENO

echo $0 >> ./ok

