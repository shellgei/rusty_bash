#!/bin/bash
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/release/rusty_bash

### BASIC BEHAVIOR ###

res=$($com <<< '(sleep 1; echo a) &')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(sleep 1; echo a) & echo b')
[ "$res" = "b
a" ] || err $LINENO

res=$($com <<< '(sleep 1; echo a) & wait ; echo b')
[ "$res" = "a
b" ] || err $LINENO

### DISPLAYING ###

#res=$($com <<< '( sleep 1 & sleep 2 ) 2>&1')
#echo $res | grep -E '^\[1\] [0-9]+$' || err $LINENO

echo OK $0
