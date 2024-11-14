#!/bin/bash -xv
# SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

### RANDOM ###

res=$($com -c '[[ "$RANDOM" -ne "$RANDOM" ]]')
[ "$?" == "0" ] || err $LINENO

res=$($com -c 'RANDOM=a ; echo "$RANDOM"')
[ "$res" != "a" ] || err $LINENO

res=$($com -c 'unset RANDOM; RANDOM=a ; echo "$RANDOM"')
[ "$res" == "a" ] || err $LINENO

### TIME ###
#
res=$($com -c '[[ 0 -eq $SECONDS ]] && sleep 1 && [[ 1 -eq $SECONDS ]]')
[[ "$?" -eq 0 ]] || err $LINENO

echo $0 >> ./ok
