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

res=$($com << 'EOF'
f () {
    COMP_LINE='cd ~/G'
    COMP_POINT=6
    local lead=${COMP_LINE:0:COMP_POINT}
    echo $lead
}
f
EOF
)
[ "$res" == "cd ~/G" ] || err $LINENO



res=$($com <<< 'rev << EOF
abc
あいう
EOF
')
[ "$res" == "cba
ういあ" ] || err $LINENO

res=$($com <<< 'A=hoge ; rev << EOF
abc
あいう
$A
EOF
')
[ "$res" == "cba
ういあ
egoh" ] || err $LINENO

res=$($com <<< 'echo `echo aaa`' )
[ "$res" = "aaa" ] || err $LINENO

res=$($com <<< 'compgen -G "/*" | wc -l' )
[ "$res" -gt 1 ] || err $LINENO

res=$($com <<< 'a=(a b); set "${a[@]}${a[@]}" ;echo $@ $#' )
[ "$res" = "a ba b 3" ] || err $LINENO

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

