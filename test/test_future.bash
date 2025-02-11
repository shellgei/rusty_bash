#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

# memo 
# sush: shopt: -po: invalid shell option name
# printf: ParseError


cd $(dirname $0)
com=../target/release/sush

res=$($com <<< 'A=(a b) ; echo ${#A[@]}')
[ "$res" -eq 2 ] || err $LINENO

res=$($com <<< 'A=(a b) ; echo "${#A[@]}"')
[ "$res" -eq 2 ] || err $LINENO

res=$($com <<< '[[ a =~ "." ]]')
[ $? -eq 1 ] || err $LINENO

echo $0 >> ./ok

res=$($com <<< 'A=1 ; echo "$((A+1))"')
[ "$res" -eq 2 ] || err $LINENO

res=$($com <<< '[[ a =~ "." ]]')
[ $? -eq 1 ] || err $LINENO

echo $0 >> ./ok
