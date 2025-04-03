#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	rm -f $tmp-*
	exit 1
}

cd $(dirname $0)
com=../target/release/sush
tmp=/tmp/$$

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

res=$($com -c 'SECONDS=-10 ; sleep 1 ; echo $SECONDS')
[[ "$res" -eq -9 ]] || err $LINENO

res=$($com <<< 'RANDOM=2 ;echo $RANDOM ; echo $RANDOM')
[ "$res" = "27297
16812" ] || err $LINENO

res=$($com <<< 'echo $((42%5))')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'echo $(( 0 ? 1 : x=3))')
[ $? -eq 1 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'declare -i i=1 j=1 ;echo $i $j ')
[ "$res" = "1 1" ] || err $LINENO

res=$($com <<< 'echo $(( 4> (2+3) ? 1 : 32))')
[ "$res" = "32" ] || err $LINENO

res=$($com <<< 'echo $(( 4>(2+3) ? 1 : 32))')
[ "$res" = "32" ] || err $LINENO

res=$($com <<< 'declare -i n; n="1+1" ; echo $n')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( n ))')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( (n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'echo $(( c=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( c=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( c+=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

rm -f $tmp-*
echo $0 >> ./ok
exit

### issue 130 ###
### input-line.sh test of Bash ###

# It works.
cat << 'EOF' > $tmp-script
read a
echo @$a
EOF
chmod +x $tmp-script
res=$(bash << EOF
$com $tmp-script
OH
EOF
)
[ "$res" = "@OH" ] || err $LINENO

# It doesn't work.
# Maybe the exec-on-close is applied to
# the file discriptor of $com << EOF. 

chmod +x $tmp-script
res=$($com << EOF
$com $tmp-script
OH
EOF
)
[ "$res" = "@OH" ] || err $LINENO

### WHY ???????????? ###

#ueda@x1gen13:~/GIT/bash_for_sush_test/sush_test$ echo "a:b:" | ( IFS=" :" read x y; echo "($x)($y)" )
#(a)(b)
#ueda@x1gen13:~/GIT/bash_for_sush_test/sush_test$ echo "a:b::" | ( IFS=" :" read x y; echo "($x)($y)" )
#(a)(b::)

