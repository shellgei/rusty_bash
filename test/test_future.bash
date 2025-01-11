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

res=$($com <<< 'set 1 2 3 4 ; echo ${@:2:2}')
[ "$res" == "2 3" ] || err $LINENO

res=$($com <<< 'set 1 2 3 4 ; echo ${@:1:2}')
[ "$res" == "1 2" ] || err $LINENO

echo $0 >> ./ok
