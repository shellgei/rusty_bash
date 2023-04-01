#!/bin/bash -xv
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/debug/rusty_bash

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

res=$($com <<< '( sleep 1 & sleep 2 ) 2>&1')
echo $res | grep -E '^\[1\] [0-9]+$' || err $LINENO

res="$($com <<< '( sleep 1 & wait ) 2>&1')"
echo $res | grep Done || err $LINENO

res="$($com <<< 'sleep 1 & sleep 1 & jobs')"
echo $res | grep '\[1\].*Running sleep 1 &.*\[2\].*Running sleep 1 &' || err $LINENO

### bg COMMAND ###

res="$($com <<< '(sleep 1 ; killall -SIGSTOP sleep ) & sleep 2 ; fg ')"
echo $res | grep -F '[2]+ Stopped sleep 2 [2]+ Done sleep 2'  || err $LINENO

res="$($com <<< 'sleep 1 || sleep 1 & jobs')"
echo $res | grep -F 'sleep 1 || sleep 1 &'  || err $LINENO

### PRIORITY ###

res="$($com <<< 'sleep 1 & sleep 2 & killall -SIGSTOP sleep ; jobs ; killall -SIGCONT sleep')"
echo $res | grep -F '[1]- Stopped sleep 1 & [2]+ Stopped sleep 2 &' || err $LINENO

res="$($com <<< 'sleep 1 & sleep 2 & killall -SIGSTOP sleep ; sleep 3 & jobs ; killall -SIGCONT sleep')"
echo $res | grep -F '[1] Stopped sleep 1 & [2]- Stopped sleep 2 & [3]+ Running sleep' || err $LINENO

echo OK $0
