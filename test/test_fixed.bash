#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

#res=$($com <<< '
#c=0
#f () {
#while [ $c -lt 3 ]; do
#        for word in a bc ; do
#                return
#        done
#        ((c++))
#done
#}
#
#f
#echo $c
#')
#[ "$res" = "0" ] || err $LINENO

cd $(dirname $0)
com=../target/release/sush

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

