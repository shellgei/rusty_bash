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
a=1
case $a in
	1) echo OK ;;
	*) 
esac
EOF
)
[ "$res" = "OK" ] || err $LINENO

res=$($com << 'EOF'
a=(aa bb cc)
echo ${!a[@]}
echo ${!a[*]}
EOF
)
[ "$res" = "0 1 2
0 1 2" ] || err $LINENO

res=$($com << 'EOF'
a=(aa bb cc)
b=("${!a[@]}")
echo ${b[0]}
b=("${!a[*]}")
echo ${b[0]}
EOF
)
[ "$res" = "0
0 1 2" ] || err $LINENO

res=$($com << 'EOF'
getopts :alF: _opt -aF : paths a:b
echo $_opt
echo $OPTARG
echo $OPTIND
getopts :alF: _opt -aF : paths a:b
echo $_opt
echo $OPTARG
echo $OPTIND
getopts :alF: _opt -aF : paths a:b
echo $_opt
echo $OPTARG
echo $OPTIND
EOF
)
[ "$res" = "a

1
F
:
3
?

3" ] || err $LINENO

res=$($com <<< 'a=A ; echo ${a:-B}' )
[ "$res" = "A" ] || err $LINENO

res=$($com <<< 'rev <(echo abc)' )
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'rev < <(echo abc)' )
[ "$res" = "cba" ] || err $LINENO

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

