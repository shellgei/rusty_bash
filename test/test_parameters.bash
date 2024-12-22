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

### RANDOM ###

res=$($com -c '[[ "$RANDOM" -ne "$RANDOM" ]]')
[ "$?" == "0" ] || err $LINENO

res=$($com -c 'RANDOM=a ; echo "$RANDOM"')
[ "$res" != "a" ] || err $LINENO

res=$($com -c 'unset RANDOM; RANDOM=a ; echo "$RANDOM"')
[ "$res" == "a" ] || err $LINENO

### TIME ###

res=$($com -c '[[ 0 -eq $SECONDS ]] && sleep 1 && [[ 1 -eq $SECONDS ]]')
[[ "$?" -eq 0 ]] || err $LINENO

res=$($com -c '[[ $(date +%s) -eq $EPOCHSECONDS ]]')
[[ "$?" -eq 0 ]] || err $LINENO

res=$($com -c 'echo $(( $EPOCHREALTIME - $(date +%s) )) | awk -F. "{print \$1}"')
[[ "$res" -eq 0 ]] || err $LINENO

### READONLY ###

res=$($com -c 'A=1 ; f () { local A ; declare -r A ; A=123 ; } ; f')
[[ "$?" -eq 1 ]] || err $LINENO

#res=$($com -c 'A=1 ; f () { local -a A ; A[1]=123 ; echo ${A[@]} ; } ; f ; echo $A')
#[[ "$res" -eq '123
#1' ]] || err $LINENO

res=$($com -c 'f () { local A ; declare -r A ; A=123 ; } ; f; A=3 ; echo $A')
[[ "$res" -eq 3 ]] || err $LINENO

res=$($com -c 'A=1 ; declare -r A ; f () { local A ; A=123 ; } ; f')
[[ "$?" -eq 1 ]] || err $LINENO

res=$($com -c 'A=1 ; declare -r A ; A=(3 4)')
[[ "$?" -eq 1 ]] || err $LINENO

### ARRAY ###
#
res=$($com <<< 'declare -a A; A[0]=bbb; echo ${A[aaa]}')
[ "$res" == "bbb" ] || err $LINENO

### ASSOCIATED ARRAY ###

res=$($com <<< 'declare -A A; A[aaa]=bbb; echo ${A[aaa]}')
[ "$res" == "bbb" ] || err $LINENO

res=$($com <<< 'declare -A A; A[aaa]=bbb ;A[ccc]=ddd ; echo ${A[@]}')
[ "$res" == "ddd bbb" -o "$res" == "bbb ddd" ] || err $LINENO

res=$($com <<< 'B=ccc; declare -A A; A[aaa]=bbb ;A[ccc]=ddd ; echo ${A[$B]}')
[ "$res" == "ddd" ] || err $LINENO

echo $0 >> ./ok
