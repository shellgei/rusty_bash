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

res=$($com <<< 'a=A ; echo ${a:-B}' )
[ "$res" = "A" ] || err $LINENO

res=$($com <<< 'rev <(echo abc)' )
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'rev < <(echo abc)' )
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< '__a=x; echo $__a ; echo $__a' )
[ "$res" = "x
x" ] || err $LINENO

res=$($com <<< '
_=aaa
echo $_
echo $_
' )
[ "$res" = "
echo" ] || err $LINENO

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

