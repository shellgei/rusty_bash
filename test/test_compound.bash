#!/bin/bash
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)

com=../target/debug/rusty_bash
tmp=/tmp/$$

### COMPOUND COMMAND ###

res=$($com <<< '(echo hoge)')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '{echo hoge; }')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '(echo hoge;echo hoge)')
[ "$res" = "hoge
hoge" ] || err $LINENO

res=$($com <<< '{echo hoge;echo hoge ; }')
[ "$res" = "hoge
hoge" ] || err $LINENO

res=$($com <<< '(echo hoge | rev;echo hoge)')
[ "$res" = "egoh
hoge" ] || err $LINENO

res=$($com <<< 'echo abc | ( echo a ; rev ) | tr -d \\n')
[ "$res" = "acba" ] || err $LINENO

res=$($com <<< '{echo hoge | rev;echo hoge ; }')
[ "$res" = "egoh
hoge" ] || err $LINENO

res=$($com <<< 'echo abc | { echo a ; rev ; } | tr -d \\n')
[ "$res" = "acba" ] || err $LINENO

res=$($com <<< '(A=B);echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< '{A=B ; };echo $A')
[ "$res" = "B" ] || err $LINENO

res=$($com <<< 'echo abc | (rev)')
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< '(echo abc) | rev')
[ "$res" = "cba" ] || err $LINENO

res=$($com << 'EOF'
(ls aaaaa) 2> /tmp/tmp_x
cat /tmp/tmp_x  | wc -l
rm /tmp/tmp_x 
EOF
)
[ "$res" -ge 1 ] || err $LINENO

res=$($com <<< '{ echo } ; }')
[ "$res" = "}" ] || err $LINENO

res=$($com <<< '( (echo hoge))')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '((echo hoge) )')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '((echo hoge))')
[ "$?" != "0" ] || err $LINENO
[ "$res" = "" ] || err $LINENO

# compound and read

res=$($com <<< 'echo あ い う | ( read b ; echo $b )')
[ "$res" = "あ い う" ] || err $LINENO

res=$($com <<< 'echo あ い う | ( read a b ; echo $b )')
[ "$res" = "い う" ] || err $LINENO

res=$($com <<< 'echo あ い う | ( read a b c ; echo $b )')
[ "$res" = "い" ] || err $LINENO

# (())

res=$($com <<< '((0));echo $?')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< '((1));echo $?')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'echo $((1+2+3))')
[ "$res" = "6" ] || err $LINENO

res=$($com <<< 'echo $((1-2+3))')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'echo $((1+2*3))')
[ "$res" = "7" ] || err $LINENO

res=$($com <<< 'echo $((1+2*-3))')
[ "$res" = "-5" ] || err $LINENO

res=$($com <<< 'echo $((1+2/3))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'echo $((-1+2/3))')
[ "$res" = "-1" ] || err $LINENO

### MULTILINE INPUT ###

res=$($com << 'EOF'
echo a \
b \
c
EOF
)
[ "$res" = "a b c" ] || err $LINENO

res=$($com << 'EOF'
ec\
ho a\
b\
c
EOF
)
[ "$res" = "abc" ] || err $LINENO

res=$($com << 'EOF'
(
echo a
echo b
)
EOF
)
[ "$res" = "a
b" ] || err $LINENO

res=$($com << 'EOF'
{
echo a
echo b
}
EOF
)
[ "$res" = "a
b" ] || err $LINENO

res=$($com << 'EOF'
(
echo a
echo b
)
EOF
)
[ "$res" = "a
b" ] || err $LINENO

res=$($com << 'EOF'
{
echo a
echo b
}
EOF
)
[ "$res" = "a
b" ] || err $LINENO

echo OK $0
