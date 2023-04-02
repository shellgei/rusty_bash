#!/bin/bash -xv
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)

com=../target/debug/rusty_bash
tmp=/tmp/$$

## GLOB FOR CASE ###

res=$($com <<< 'glob_test "a*" abcde')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "a*" z')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[abc]" a')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[!abc]" a')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[^abc]" a' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[abc][bcd][xy]" adx')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[abc][bcd][!xy]" adx' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[!abc!]" "!"' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[a-z]" "b"')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[!a-c]" "b"')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'echo a || echo b || echo c')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< 'echo a && echo b || echo c')
[ "$res" = "a
b" ] || err $LINENO

res=$($com <<< 'echo a || echo b && echo c')
[ "$res" = "a
c" ] || err $LINENO

### CASE ###

res=$($com <<< 'case $- in *x*) echo x ;; *) echo no ;; esac')
[ "$res" = "no" ] || err $LINENO

res=$($com <<< 'case $- in *x*) ;; *) echo no ;; esac')
[ "$res" = "no" ] || err $LINENO

res=$($com -x <<< 'case $- in *x*) echo x ;; *) echo no ;; esac')
[ "$res" = "x" ] || err $LINENO

res=$($com <<< 'A=hoge ; case $A in *x*|*h*) echo aaa ;; *) echo no ;; esac')
[ "$res" = "aaa" ] || err $LINENO

res=$($com << 'EOF'
case xterm-color in
    xterm-color|*-256color) color_prompt=yes;;
esac
echo $color_prompt
EOF
)
[ "$res" = "yes" ] || err $LINENO

res=$($com << 'EOF'
case $- in 
	*x*) echo x ;;
	*) echo no ;;
esac
EOF
)
[ "$res" = "no" ] || err $LINENO

res=$($com <<< 'case a in b) echo A ;& b) echo B ;; c) echo C ;; esac' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'case a in a) echo A ;& b) echo B ;; c) echo C ;; esac' )
[ "$res" = "A
B" ] || err $LINENO

res=$($com <<< 'case a in a) echo A ;;& b) echo B ;; a) echo C ;; esac' )
[ "$res" = "A
C" ] || err $LINENO

res=$($com <<< 'case a in a) echo A ;& b) echo B ;& c) echo C ;; esac' )
[ "$res" = "A
B
C" ] || err $LINENO

echo OK $0
