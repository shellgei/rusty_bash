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

### REDIRECTS ###

# <, >, >>

res=$($com <<< 'cat < /etc/passwd | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'cat < /etc/passwd > /tmp/rusty_bash1 ; cat /tmp/rusty_bash1 | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'echo a > /tmp/rusty_bash1 ; echo b >> /tmp/rusty_bash1; cat /tmp/rusty_bash1')
[ "$res" = "a
b" ] || err $LINENO

# non-fork redirects

res=$($com <<< '
	cd /etc/
	cd /tmp
	cd - > /tmp/rusty_bash1
	cd - > /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
/tmp" ] || err $LINENO

res=$($com <<< '
	cd /etc/
	cd /tmp
	{ cd - ; cd - ; /bin/echo a; } > /tmp/rusty_bash1
	{ cd - ; } > /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
a
/etc
/etc" ] || err $LINENO

# 2>, 2>>

res=$($com <<< '
	ls /aaaa 2> /tmp/rusty_bash_$$
	ls /bbbb 2>> /tmp/rusty_bash_$$
	cat /tmp/rusty_bash_$$ | grep ls | wc -l
	' | tr -dc 0-9)
[ "$res" = "2" ] || err $LINENO

# &>

res=$($com <<< 'ls /etc/passwd aaaa &> /tmp/rusty_bash_o; cat /tmp/rusty_bash_o | wc -l | tr -dc 0-9')
[ "$res" == "2" ] || err $LINENO

# &> for non-fork redirects

res=$($com <<< '
	{ ls /etc/passwd aaaa ; } &> /tmp/rusty_bash_o
	cat /tmp/rusty_bash_o | wc -l | tr -dc 0-9')
[ "$res" == "2" ] || err $LINENO

res=$(LANG=C $com <<< '
	{ ls /etc/passwd aaaa ; } &> /tmp/rusty_bash_o
	cat /tmp/rusty_bash_o | wc -l
	#ちゃんと標準出力が原状復帰されているか調査
	{ ls /etc/passwd ; }
	{ ls aaaa ; } 2> /tmp/rusty_bash_o2
	cat /tmp/rusty_bash_o2 | wc -l
	' | tr -d '[:blank:]')
[ "$res" == "2
/etc/passwd
1" ] || err $LINENO

res=$($com <<< '
	cd /etc/
	cd /tmp
	{ cd - ; cd - ; /bin/echo a; } &> /tmp/rusty_bash1
	{ cd - ; } &> /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
a
/etc
/etc" ] || err $LINENO

# >&

b=$(ls aaaaaaaaaaaaaa 2>&1 | wc -l)
res=$($com <<< 'ls aaaaaaaaaaaaaa 2>&1 | wc -l')
[ "$b" == "$res" ] || err $LINENO

#res=$($com <<< 'pwd 200>&100')  <- not passed on macOS of GitHub Actions, 20241019
#[ "$?" == "1" ] || err $LINENO

#res=$($com <<< 'ls 200>&100')  <- not passed on macOS of GitHub Actions, 20241019
#[ "$?" == "1" ] || err $LINENO

# with expansion

res=$($com <<< 'echo a > {a,b}' 2>&1)
[ "$?" == "1" ] || err $LINENO
[[ "$res" =~ ambiguous ]] || err $LINENO

# herestring

res=$($com <<< 'rev <<< あいう')
[ "$res" == "ういあ" ] || err $LINENO

res=$($com <<< 'cat <<< $(seq 3)')
[ "$res" == "1
2
3" ] || err $LINENO

if [ "$(uname)" = "Linux" ] ; then
	res=$($com <<< 'cat <<< $(seq 3000) | wc -l')
	[ "$res" == "3000" ] || err $LINENO

	res=$($com <<< 'cat <<< $(aaa) | wc -l')
	[ "$res" == "1" ] || err $LINENO
fi

# here documents

res=$($com <<< 'rev << EOF
abc
あいう
EOF
')
[ "$res" == "cba
ういあ" ] || err $LINENO

res=$($com <<< 'A=hoge ; rev << EOF
abc
あいう
$A
EOF
')
[ "$res" == "cba
ういあ
egoh" ] || err $LINENO


echo $0 >> ./ok
