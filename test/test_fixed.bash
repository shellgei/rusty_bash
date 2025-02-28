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

res=$($com <<< 'cd -- ""' )
[ $? -eq 0 ] || err $LINENO

if [ ! -e ~/tmp/a/b ] ; then 
	res=$($com <<< '
	mkdir -p ~/tmp/a/b
	touch ~/tmp/a/b/c
	compgen -f -X "" -- "~/tmp/a/b/c"
	rm ~/tmp/a/b/c
	rmdir -p ~/tmp/a/b
	rmdir -p ~/tmp/a
	' ) 2> /dev/null
	[ "$res" = "~/tmp/a/b/c" ] || err $LINENO
fi

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

