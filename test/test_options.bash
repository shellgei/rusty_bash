#!/bin/bash -xv
# SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

### -c

res=$($com -c "echo a")
[ "$?" == "0" ] || err $LINENO
[ "$res" == "a" ] || err $LINENO

res=$($com -c "ech a")
[ "$?" == "127" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$(echo abc | $com -c "rev")
[ "$res" == "cba" ] || err $LINENO

res=$($com -c -)
[[ "$?" -eq 2 ]] || err $LINENO

res=$($com -c 'echo $@' a b c)
[ "$res" == "b c" ] || err $LINENO

res=$($com -c 'echo $0' a b c)
[ "$res" == "a" ] || err $LINENO

### -e

res=$($com <<< 'set -e ; false ; echo NG')
[ "$res" != "NG" ] || err $LINENO

res=$($com <<< 'set -e ; false | true ; echo OK')
[ "$res" == "OK" ] || err $LINENO

res=$($com <<< 'set -e ; ( false ) ; echo NG')
[ "$res" != "NG" ] || err $LINENO

res=$($com <<< 'set -e ; false || echo OK')
[ "$res" == "OK" ] || err $LINENO

res=$($com <<< 'set -e ; false || false ; echo NG')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'set -e ; while false ; do echo NG ; done ; echo OK')
[ "$res" == "OK" ] || err $LINENO

res=$($com <<< 'set -o pipefail; ls aaaa | false | true')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'set -o pipefail; set -e; false | true ; echo NG')
[ "$res" == "" ] || err $LINENO

### -B

res=$($com <<< 'set +B; echo {a,b}')
[ "$res" == "{a,b}" ] || err $LINENO

### noglob

res=$($com <<< 'set -o noglob; echo /etc/*')
[ "$res" = "/etc/*" ] || err $LINENO


echo $0 >> ./ok
