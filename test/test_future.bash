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

#$ echo 'aaa\
#bbb' | ( read -r a ; echo $a )
#aaa\
#ueda@x1gen13:~$ echo 'aaa\
#bbb' | ( read a ; echo $a )
#aaabbb


res=$($com <<< 'cur="~" ; [[ $cur == \~* ]]' )
[ "$?" -eq 0 ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

res=$($com <<< 'printf -v REPLY %q /l; echo $REPLY')
[ "$res" = "/l" ] || err $LINENO

res=$($com <<< '[[ a =~ "." ]]')
[ $? -eq 1 ] || err $LINENO

echo $0 >> ./ok
