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

res=$($com <<< 'seq 3 |    
#コメントだよ
    #こめんとだよ
	  tac | head -n 1')
[ "$res" = "3" ] || err $LINENO

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

echo OK $0
