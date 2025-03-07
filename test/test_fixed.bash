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

res=$($com <<< 'declare -A a; echo ${a[aaa]}')
[ "$?" = "0" ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'a=(); a=("${a[@]}"); echo ${#a[@]}')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'a=(1 2 3); a=("${a[@]:3}"); echo ${#a[@]}')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'cur=r ;echo ${cur//[[:space:]]/}')
[ "$res" = "r" ] || err $LINENO

res=$($com <<< 'a=aba; echo ${a^^[ac]}' )
[ "$res" = "AbA" ] || err $LINENO

res=$($com <<< 'a=あacaba; echo ${a^^[ac]}' )
[ "$res" = "あACAbA" ] || err $LINENO

res=$($com <<< 'a=あacaba; echo ${a^^[cあ]}' )
[ "$res" = "あaCaba" ] || err $LINENO

res=$($com <<< 'a=あAcabA; echo ${a,,[Aあ]}' )
[ "$res" = "あacaba" ] || err $LINENO

res=$($com <<< 'a=あAcabA; echo ${a~~[Aaあ]}' )
[ "$res" = "あacAba" ] || err $LINENO

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

