#!/bin/bash
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/debug/rusty_bash

### UNEXPECTED TOKEN ###

res=$($com <<< '( )' 2>&1)
[ "$?" = "2" ] || err $LINENO
[ "$res" = "Unexpected token: )" ] || err $LINENO

res=$($com <<< '( }' 2>&1)
[ "$?" = "2" ] || err $LINENO
[ "$res" = "Unexpected token: }" ] || err $LINENO

res=$($com <<< '}' 2>&1)
[ "$?" = "2" ] || err $LINENO
[ "$res" = "Unexpected token: }" ] || err $LINENO

res=$($com <<< ')' 2>&1)
[ "$?" = "2" ] || err $LINENO
[ "$res" = "Unexpected token: )" ] || err $LINENO

echo OK $0
