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

res=$($com <<< '(echo hoge
echo fuge)')
[ "$res" = "hoge
fuge" ] || err $LINENO

res=$($com <<< '(echo hoge
echo fuge)
(    
echo b)
  (
echo c
)')
[ "$res" = "hoge
fuge
b
c" ] || err $LINENO

### IRREGULAR COMMAND TEST ###

res=$($com <<< 'eeeeeecho hoge')
[ "$?" = 127 ] || err $LINENO

#res=$($com <<< '(echo hoge; false)')
#[ "$?" = 1 ] || err $LINENO

echo OK $0
