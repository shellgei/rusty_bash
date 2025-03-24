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

res=$($com <<< 'echo /bin/?' | grep -F '/bin/[')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo /*' | grep '/etc')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo ~+/*' | grep '*')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'echo ~/*' | grep -F '/.')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'echo ~/.*' | grep -F '/.')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo /etc*/' | grep -F '/etc/')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo .*' | grep -F './.')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'echo ./*' | grep -F './')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo *"$PATH"')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo /*"b"*' | grep -F '*')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< "echo /*'b'*" | grep -F '*')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo /"*"' | grep -F '*')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'cd /etc ; echo *.conf/' | grep -F '*')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo @(あ|{い,う,})')
[ "$res" == "@(あ|い) @(あ|う) @(あ|)" ] || err $LINENO

res=$($com <<< 'echo \/e\tc/* | grep -F "*"')
[ $? -eq 1 ] || err $LINENO

if [ "$(uname)" = Linux ] ; then
	res=$($com <<< 'touch /tmp/2 ; echo /tmp/[1-5]' | grep 2)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/1 ; echo /tmp/[5-1]' | grep -- -)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/5 ; echo /tmp/[5-1]' | grep -- -)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/C ; echo /tmp/[A-D]' | grep C)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/c ; echo /tmp/[a-d]' | grep c)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/9 ; echo /tmp/[1-59]' | grep 9)
	[ "$?" == "0" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/{1..9} ; echo /tmp/[1-37-9]')
	[ "$res" == "/tmp/1 /tmp/2 /tmp/3 /tmp/7 /tmp/8 /tmp/9" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/{1..9} ; ls /tmp/[!1-37-9] | grep "/tmp/[1-9]" | xargs')
	[ "$res" == "/tmp/4 /tmp/5 /tmp/6" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/{1..9} ; echo /tmp/[1-]')
	[ "$res" == "/tmp/1" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/{1..9} ; ls /tmp/[!1-] | grep ^/tmp/1$')
	[ "$?" == "1" ] || err $LINENO
	
	res=$($com <<< 'touch /tmp/{1..9} ; ls /tmp/[!1-] | grep ^/tmp/5$')
	[ "$?" == "0" ] || err $LINENO
fi 

echo $0 >> ./ok
