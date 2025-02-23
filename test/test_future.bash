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

res=$($com <<< 'a= ; echo ${a[@]}')
[ "$?" -eq 0 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< '[[ "a\ b" == "a\ b" ]]; echo $?')
[ "$res" = "0" ] || err $LINENO


res=$($com <<< 'a=" a  b  c "; echo $a; IFS= ; echo $a')
[ "$res" = "a b c
 a  b  c " ] || err $LINENO

res=$($com <<< 'a="@a@b@c@"; IFS=@ ; echo $a@')
[ "$res" = " a b c @" ] || err $LINENO

res=$($com <<< 'a="@a@b@c@"; IFS=@ ; echo $a')
[ "$res" = " a b c" ] || err $LINENO

res=$($com << 'EOF'
IFS='
'
set a '1
2
3'

eval "$1=(\$2)"
echo ${#a[@]}

IFS=
eval "$1=(\$2)"
echo ${#a[@]}
EOF
)
[ "$res" = "3
1" ] || err $LINENO

res=$($com <<< 'a=abca ; echo @${a//a}@')
[ "$res" = "@bc@" ] || err $LINENO

res=$($com <<< 'a=abca ; echo @${a//a/}@')
[ "$res" = "@bc@" ] || err $LINENO

res=$($com <<< 'a=" " ; echo @${a/[[:space:]]/}@')
[ "$res" = "@@" ] || err $LINENO

res=$($com <<< 'a="  " ; echo @${a/[[:space:]]/}@')
[ "$res" = "@ @" ] || err $LINENO

res=$($com <<< 'a="  " ; echo @${a//[[:space:]]/}@')
[ "$res" = "@@" ] || err $LINENO

res=$($com <<< 'a=(a b) ; echo ${a+"${a[@]}"}')
[ "$res" = "a b" ] || err $LINENO

res=$($com << 'EOF'
cur='~'
[[ $cur == '~' ]]
EOF
)
[ "$?" -eq 0 ] || err $LINENO

res=$($com << 'EOF'
[[ ~ == '~' ]]
EOF
)
[ "$?" -eq 1 ] || err $LINENO

res=$($com << 'EOF'
cur="~"
[[ $cur == \~* ]]
EOF
)
[ "$?" -eq 0 ] || err $LINENO

res=$($com << 'EOF'
_cur=a
b=(${_cur:+-- "$_cur"})
echo ${b[0]}
echo ${b[1]}
EOF
)
[ "$res" = "--
a" ] || err $LINENO

echo $0 >> ./ok
exit


res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

res=$($com <<< 'printf -v REPLY %q /l; echo $REPLY')
[ "$res" = "/l" ] || err $LINENO

res=$($com <<< '[[ a =~ "." ]]')
[ $? -eq 1 ] || err $LINENO

