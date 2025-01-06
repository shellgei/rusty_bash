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

res=$($com <<< 'b[a]=123; echo ${b[0]}')
[ "$res" == "123" ] || err $LINENO

res=$($com <<< 'b[a]=123; a=1; echo ${b[a]}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< "b[a]=123; echo ${b['a']}")
[ "$?" -eq 1 ] || err $LINENO

res=$($com <<< 'a=3; b[a]=123; echo ${b[3]}')
[ "$res" == "123" ] || err $LINENO

res=$($com << 'EOF'
declare -A c
c['a']=abc
echo ${c[@]}
echo ${c[a]}
echo ${c[0]}
echo ${c['a']}
EOF
)
[ "$res" == "abc
abc

abc" ] || err $LINENO

echo $0 >> ./ok
