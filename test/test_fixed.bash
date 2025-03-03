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

#res=$($com <<< 'a=aaa; echo ${a^^}' )
#[ "$res" = "AAA" ] || err $LINENO

res=$($com << 'EOF'
A=def
B=${A-${A-}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set 0
declare -a ASSOC
ASSOC[0]=def
B=${ASSOC[$1]-${ASSOC[$1]-}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set vim
declare -A ASSOC
ASSOC[vim]=def
B=${ASSOC[$1]-}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set vim
declare -A ASSOC
ASSOC[vim]=def
B=${ASSOC[$1]-${ASSOC[$1]}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set vim
declare -A ASSOC
ASSOC[vim]=def
B=${ASSOC[$1]-${ASSOC[$1]-}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set vim
declare -A ASSOC
ASSOC["vim"]=def
B=${ASSOC[$1]-${ASSOC[$1]-}}
echo $B
EOF
)
[ "$res" == "def" ] || err $LINENO

res=$($com << 'EOF'
set vim
declare -A ASSOC
ASSOC["vim"]=def
echo ${1##*/}
B=${ASSOC[${1##*/}]-${ASSOC[${1##*/}]-}}
echo $B
echo ${ASSOC[${1##*/}]-}
EOF
)
[ "$res" == "vim
def
def" ] || err $LINENO

res=$($com <<< 'compgen -d -- "~/" | wc -l' )
[ "$res" != "0" ] || err $LINENO

res=$($com <<< '[[ -d == -d ]]' )
[ $? -eq 2 ] || err $LINENO

res=$($com <<< 'a="-d" ; [[ $a == -d ]]' )
[ $? -eq 0 ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); echo ${a[*]:2}' )
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); echo ${a[@]:1+1}' )
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); i=0; j=1; echo ${a[@]:i+j}' )
[ "$res" = "bb cc" ] || err $LINENO

res=$($com <<< 'a=(aa bb cc); i=0; j=1; echo ${a[@]:0:1}' )
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'a=3; b=4; echo $((a-=b))' )
[ "$res" = "-1" ] || err $LINENO

res=$($com <<< 'echo $((a-=b))' )
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'if [ a = b ] ; then echo a ; fi' )
[ "$?" -eq "0" ] || err $LINENO

res=$($com << 'EOF'
echo 'echo $1' > /tmp/$$-tmp
source /tmp/$$-tmp aaa
EOF
)
[ "$res" = "aaa" ] || err $LINENO

res=$($com << 'EOF'
export A=1
bash -c 'echo $A'
EOF
)
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'A=1; readonly A ; A=2; echo $A' )
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'shopt -s nullglob ; echo aaaaaa*' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'shopt -s nullglob ; echo aaaaaa*; shopt -u nullglob ; echo aaaaaa*' )
[ "$res" = "
aaaaaa*" ] || err $LINENO

echo $0 >> ./ok
exit

### fixed in future ###

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

