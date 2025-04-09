#!/bin/bash -xv
# SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/release/sush

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

res=$($com <<< 'a=(a ""); set ${a[@]+"${a[@]}"}; echo $# ')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'A=1; echo ${A/#//d}')
[ "$res" = "/d1" ] || err $LINENO

res=$($com <<< 'A=1; echo ${A/#1//d}')
[ "$res" = "/d" ] || err $LINENO

res=$($com <<< 'A= ; echo ${A/#//d}')
[ "$res" = "/d" ] || err $LINENO

res=$($com <<< 'echo ${A/#//d}')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=1; echo ${A/%//d}')
[ "$res" = "1/d" ] || err $LINENO

res=$($com <<< 'A= ; echo ${A/%//d}')
[ "$res" = "/d" ] || err $LINENO

res=$($com <<< 'echo ${A/%//d}')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'paths=(a b c); echo ${paths[@]/%//d}')
[ "$res" = "a/d b/d c/d" ] || err $LINENO

res=$($com << 'EOF'
set 0
declare -a ASSOC
ASSOC[0]=def
B=${ASSOC[$1]-${ASSOC[$1]-}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); echo ${a[*]:2}' )
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); echo ${a[@]:1+1}' )
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); i=0; j=1; echo ${a[@]:i+j}' )
[ "$res" = "bb cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); i=0; j=1; echo ${a[@]:0:1}' )
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'a=3; b=4; echo $((a-=b))' )
[ "$res" = "-1" ] || err $LINENO

res=$($com <<< 'echo $((a-=b))' )
[ "$res" = "0" ] || err $LINENO

echo $0 >> ./ok
