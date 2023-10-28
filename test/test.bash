#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cargo build || err $LINENO

cd $(dirname $0)
com=../target/debug/sush

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ] || err $LINENO

### COMPOUND COMMAND TEST ###

res=$($com <<< '(echo hoge; echo fuge)')
[ "$res" = "hoge
fuge" ] || err $LINENO

res=$($com <<< '(echo a; (echo b ; echo c) )')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '(
echo a; (echo b ; 
echo c) )')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '   (

echo a; (echo b ; 

echo c) )   ')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '   (#aaaa

echo a; (echo b ;  #bbb

echo c) )   ')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '(
echo a; (echo b ; 
')
[ "$?" = "2" ] || err $LINENO

res=$($com <<< '(echo hoge; false)')
[ "$?" = 1 ] || err $LINENO

res=$($com <<< 'cd / ; (cd /etc); pwd')
[ "$res" = / ] || err $LINENO

res=$($com <<< 'cd / ; { cd /etc ; } ; pwd')
[ "$res" = /etc ] || err $LINENO

res=$($com <<< '( )')
[ "$?" = 2 ] || err $LINENO

res=$($com <<< '( echo a ; }')
[ "$?" = 2 ] || err $LINENO

res=$($com <<< '{ echo a ; }')
[ $res = "a" ] || err $LINENO

res=$($com <<< '{ echo a ; echo b ;}')
[ "$res" = "a
b" ] || err $LINENO

res=$($com <<< '{ echo a ; (echo b ; echo c) ;}')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '{ echo } ; }')
[ "$res" = "}" ] || err $LINENO

res=$($com <<< '{ echo a }')
[ "$?" = 2 ] || err $LINENO

### IRREGULAR COMMAND TEST ###

res=$($com <<< 'eeeeeecho hoge')
[ "$?" = 127 ] || err $LINENO

### PIPELINE ###

res=$($com <<< 'seq 3 | tac | head -n 1')
[ "$res" = "3" ] || err $LINENO

res=$($com <<< 'seq 3 |
	tac | head -n 1')
[ "$res" = "3" ] || err $LINENO

res=$($com <<< 'seq 3 |    

	  tac | head -n 1')
[ "$res" = "3" ] || err $LINENO

res=$($com <<< 'seq 3 |  #コメントだよ

#コメントだよ
    #こめんとだよ

	  tac | head -n 1')
[ "$res" = "3" ] || err $LINENO

res=$($com <<< 'seq 3 |   | head -n 1')
[ "$?" = "2" ] || err $LINENO

### COMMENT ###

res=$($com <<< 'echo a #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '
#comment comment
   #comment comment
echo a #aaaaa
#comment comment
')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a) #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a)#aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '{ echo a; }#aaaaa')
[ "$res" != "a" ] || err $LINENO

res=$($com <<< '{ echo a; } #aaaaa')
[ "$res" = "a" ] || err $LINENO

### NEW LINE ###

res=$($com <<< 'e\
c\
ho hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'e\
c\
ho \
hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge |\
rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< 'echo hoge |\
& rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< ' (seq 3; seq 3) | grep 3 | wc -l')
[ "$res" = "2" ] || err $LINENO

### REDIRECTS ###

res=$($com <<< 'cat < /etc/passwd | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'cat < /etc/passwd > /tmp/rusty_bash1 ; cat /tmp/rusty_bash1 | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'echo a > /tmp/rusty_bash1 ; echo b >> /tmp/rusty_bash1; cat /tmp/rusty_bash1')
[ "$res" = "a
b" ] || err $LINENO

res=$($com <<< '
	cd /etc/
	cd /tmp
	cd - > /tmp/rusty_bash1
	cd - > /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd')
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
	pwd')
[ "$res" = "/etc
/tmp
a
/etc
/etc" ] || err $LINENO

res=$($com <<< '
	ls /aaaa 2> /tmp/rusty_bash
	ls /bbbb 2>> /tmp/rusty_bash
	cat /tmp/rusty_bash | grep ls | wc -l
	')
[ "$res" = "2" ] || err $LINENO

#res=$($com <<< 'cat <' 2>&1)
#[ "$?" == "2" ] || err $LINENO
#[ "$res" == 'bash: syntax error near unexpected token: `newline'\''' ] || err $LINENO

echo OK $0
