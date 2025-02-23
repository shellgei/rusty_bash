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


### ARRAY ###

res=$($com <<< 'declare -a A; A[0]=bbb; echo ${A[aaa]}')
[ "$res" == "bbb" ] || err $LINENO

### INVALID REF ###

res=$($com <<< 'a= ; echo ${a[@]}')
[ "$?" -eq 0 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

### ASSOCIATED ARRAY ###

res=$($com <<< 'declare -A A; A[aaa]=bbb; echo ${A[aaa]}')
[ "$res" == "bbb" ] || err $LINENO

res=$($com <<< 'declare -A A; A[aaa]=bbb ;A[ccc]=ddd ; echo ${A[@]}')
[ "$res" == "ddd bbb" -o "$res" == "bbb ddd" ] || err $LINENO

res=$($com <<< 'B=ccc; declare -A A; A[aaa]=bbb ;A[ccc]=ddd ; echo ${A[$B]}')
[ "$res" == "ddd" ] || err $LINENO

res=$($com <<< 'declare -a arr ; arr=bbb ; echo ${arr[0]}')
[ "$res" == "bbb" ] || err $LINENO

### FUNCNAME ###

res=$($com <<< 'f(){ g () { echo ${FUNCNAME[@]} ;} ; g ;} ; f')
[ "$res" == "g f" ] || err $LINENO

### INDIRECT EXPANSION ###

res=$($com -c 'A=B; B=100; echo ${!A}')
[[ "$res" == 100 ]] || err $LINENO

res=$($com -c 'set a b ; A=1;echo ${!A}')
[[ "$res" == a ]] || err $LINENO

res=$($com -c ' A=@@; echo ${!A}')
[[ "$?" -eq 1 ]] || err $LINENO

res=$($com <<< 'a=(aaa bbb); bbb=eeee ; echo ${!a[1]}')
[ "$res" = "eeee" ] || err $LINENO

res=$($com <<< 'a=(aaa bbb); bbb=eeee ; echo ${!a[1]/ee/bb}')
[ "$res" = "bbee" ] || err $LINENO

res=$($com <<< 'a=(aaa bbb[2]); bbb[2]=eeee ; echo ${!a[1]}')
[ "$res" = "eeee" ] || err $LINENO

### PARTIAL POSITION PARAMETER ###

res=$($com <<< 'set 1 2 3 4 ; echo ${@:2:2}')
[ "$res" == "2 3" ] || err $LINENO

res=$($com <<< 'set 1 2 3 4 ; echo ${@:1:2}')
[ "$res" == "1 2" ] || err $LINENO

res=$($com <<< 'B=(1 2 3) ; A=("${B[2]}") ; echo ${A[0]}')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'set a b ; A=("${@}") ; echo ${A[1]}')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'set a b ; A=("${@:1}") ; echo ${A[0]}')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'set a b c ; A=("${@:1:1}") ; echo ${A[0]}')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'A=(a b) ; echo ${#A[@]}')
[ "$res" -eq 2 ] || err $LINENO

res=$($com <<< 'A=(a b) ; echo "${#A[@]}"')
[ "$res" -eq 2 ] || err $LINENO

res=$($com <<< 'b=1 ; f () { echo $# ; echo $1 ; } ; f ${b+"$b"}')
[ "$res" = "1
1" ] || err $LINENO

res=$($com <<< 'b= ; f () { echo $# ; echo $1 ; } ; f ${b+"$b"}')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'b=() ; f () { echo $# ; echo $1 ; } ; f ${b[@]+"aaa"}')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'b=() ; f () { echo $# ; echo $1 ; } ; f ${b[@]+"${b[@]}"}')
[ "$res" = "0" ] || err $LINENO

### IFS ###

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


echo $0 >> ./ok
