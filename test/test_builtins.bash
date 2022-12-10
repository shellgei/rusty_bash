#!/bin/bash
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)

com=../target/release/rusty_bash

### BUILTIN COMMAND ###

cat << EOF > /tmp/.rusty_bash
A=B
EOF
res=$($com <<< 'source /tmp/.rusty_bash ; echo $A')
[ "$res" = "B" ] || err $LINENO

res=$($com <<< 'set a b c ; shift; echo $1')
[ "$res" = "b" ] || err $LINENO

res=$($com <<< 'set a b c ; shift 3; echo $? ; echo $1')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'set a b c ; shift 4; echo $? ; echo $1')
[ "$res" = "1
a" ] || err $LINENO

# export

res=$($com <<< 'HOGE=A;export HOGE;printenv HOGE')
[ "$res" = "A" ] || err $LINENO

res=$($com <<< 'export HOGE=A;printenv HOGE')
[ "$res" = "A" ] || err $LINENO

echo OK $0
