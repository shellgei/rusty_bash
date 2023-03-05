#!/bin/bash
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)

com=../target/debug/rusty_bash

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

# eval

res=$($com <<< 'eval echo hello')
[ "$res" = "hello" ] || err $LINENO

res=$($com <<< "eval 'echo $(echo a b c)'")
[ "$res" = "a b c" ] || err $LINENO

res=$($com <<< "eval 'echo $(seq 3)'")
[ "$res" = "1 2 3" ] || err $LINENO

res=$($com <<< 'eval "echo $(seq 3)"')
[ "$?" = "127" ] || err $LINENO

res=$($com <<< 'eval "echo $(echo a b c)" "echo $(echo a b c)"' )
[ "$res" = "a b c echo a b c" ] || err $LINENO

res=$($com <<< 'eval "echo $(echo a b c);" "echo $(echo a b c)"' )
[ "$res" = "a b c
a b c" ] || err $LINENO


echo OK $0
