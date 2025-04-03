#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/release/sush

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

res=$($com <<< 'echo {a,b}c')
[ "$res" == "ac bc" ] || err $LINENO

res=$($com <<< 'echo c{a,b}')
[ "$res" == "ca cb" ] || err $LINENO

res=$($com <<< 'echo {{a},b}')
[ "$res" == "{a} b" ] || err $LINENO

res=$($com <<< 'echo {a,{b},c}')
[ "$res" == "a {b} c" ] || err $LINENO

res=$($com <<< 'echo {a,b,c{d,e}f,g{h,i{j,k}}}')
[ "$res" == "a b cdf cef gh gij gik" ] || err $LINENO

res=$($com <<< 'echo {a,b,c{d,e}f,g{h,i{j,k}}')
[ "$res" == "{a,b,cdf,gh {a,b,cdf,gij {a,b,cdf,gik {a,b,cef,gh {a,b,cef,gij {a,b,cef,gik" ] || err $LINENO

res=$($com <<< 'echo c{a,b')
[ "$res" == "c{a,b" ] || err $LINENO

res=$($com <<< 'echo c{a,b,')
[ "$res" == "c{a,b," ] || err $LINENO

res=$($com <<< 'echo {{a,あいうえお@},{c,d},')
[ "$res" == "{a,c, {a,d, {あいうえお@,c, {あいうえお@,d," ] || err $LINENO

res=$($com <<< 'echo {{a,b},{c,d')
[ "$res" == "{a,{c,d {b,{c,d" ] || err $LINENO

res=$($com <<< 'echo {{a,b,{c,')
[ "$res" == "{{a,b,{c," ] || err $LINENO

res=$($com <<< 'echo {a}')
[ "$res" == "{a}" ] || err $LINENO

res=$($com <<< 'echo {a,}')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'echo {a,b,}')
[ "$res" == "a b" ] || err $LINENO

res=$($com <<< 'echo {a,b,}c')
[ "$res" == "ac bc c" ] || err $LINENO

res=$($com <<< 'echo {}')
[ "$res" == "{}" ] || err $LINENO

res=$($com <<< 'echo {,}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo {,,}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo a{,,}b')
[ "$res" == "ab ab ab" ] || err $LINENO

res=$($com <<< 'echo {')
[ "$res" == "{" ] || err $LINENO

res=$($com <<< 'echo }')
[ "$res" == "}" ] || err $LINENO

res=$($com <<< 'echo {a,}{b,}')
[ "$res" == "ab a b" ] || err $LINENO

res=$($com <<< 'echo {d}d{},dba}')
[ "$res" == "d}d{} dba" ] || err $LINENO
#[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo {}a,b}')
[ "$res" == "{}a,b}" ] || err $LINENO

res=$($com <<< 'echo c{}a,b}')
[ "$res" == "c}a cb" ] || err $LINENO

res=$($com <<< 'echo {,}{}a,b}')
[ "$res" == "{}a,b} {}a,b}" ] || err $LINENO

res=$($com <<< 'echo a{}},b}')
[ "$res" == "a}} ab" ] || err $LINENO

res=$($com <<< 'echo $${a,b} | sed -E "s/[0-9]+/num/g"' )
[ "$res" == "num{a,b}" ] || err $LINENO

res=$($com <<< 'echo $${a,{b,c},d} | sed -E "s/[0-9]+/num/g"')
[ "$res" == "num{a,{b,c},d}" ] || err $LINENO

res=$($com <<< 'echo あ{a,b}{},c}')
[ "$res" == "あa{},c} あb{},c}" ] || err $LINENO

res=$($com <<< 'echo あ{a,b}d{},c}')
[ "$res" == "あad} あadc あbd} あbdc" ] || err $LINENO

# brace range

res=$($com <<< 'echo a{1..3}b')
[ "$res" == "a1b a2b a3b" ] || err $LINENO

res=$($com <<< 'echo {1..-1}')
[ "$res" == "1 0 -1" ] || err $LINENO

res=$($com <<< 'echo {1..1}')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo {あ..あ}')
[ "$res" == "あ" ] || err $LINENO

res=$($com <<< 'echo {あ..お}')
[ "$res" == "あ ぃ い ぅ う ぇ え ぉ お" ] || err $LINENO

res=$($com <<< 'echo {お..あ}')
[ "$res" == "お ぉ え ぇ う ぅ い ぃ あ" ] || err $LINENO

res=$($com <<< 'echo {0..\,}')
[ "$res" == "0 / . - ," ] || err $LINENO

res=$($com <<< 'echo {0..,}')
[ "$res" == "0.." ] || err $LINENO

res=$($com <<< 'echo {0..10..4}')
[ "$res" == "0 4 8" ] || err $LINENO

res=$($com <<< 'echo {0..10..-4}')
[ "$res" == "0 4 8" ] || err $LINENO

res=$($com <<< 'echo {0..3..0}')
[ "$res" == "0 1 2 3" ] || err $LINENO

res=$($com <<< 'echo {0..3.0}')
[ "$res" == "{0..3.0}" ] || err $LINENO

res=$($com <<< 'echo {1..2}{1..2}')
[ "$res" == "11 12 21 22" ] || err $LINENO

res=$($com <<< 'echo {2147483650..2147483655}')
[ "$res" = "2147483650 2147483651 2147483652 2147483653 2147483654 2147483655" ] || err $LINENO

echo $0 >> ./ok
