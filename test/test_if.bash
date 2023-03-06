#!/bin/bash -xv
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)

com=../target/debug/rusty_bash
tmp=/tmp/$$

### IF COMPOUND ###

res=$($com <<< 'if [ "a" == "a" ] ; then echo aa; fi')
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; fi' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; fi' || echo x)
[ "$res" = "x" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo a ; fi ; if [ "b" == "b" ] ; then echo bb ; fi')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'echo a | if [ "$(cat)" == "a" ] ; then echo aa; fi')
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'echo a | if [ "$(cat)" == "a" ] ; then echo abc; fi | rev')
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "c" ] ; then echo bb; else echo cc; fi')
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'if [ "a" == "a" ] ; then echo aa; elif [ "b" == "c" ] ; then echo bb; else echo cc; fi')
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "b" ] ; then echo bb; else echo cc; fi')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "b" ] ; then echo bb; fi')
[ "$res" = "bb" ] || err $LINENO

res=$($com << 'EOF'
if
false
then
echo hoge
elif
false
then
echo hoge
elif
false
then
echo hoge
else
echo true
fi
EOF
)
[ "$res" = "true" ] || err $LINENO

res=$($com << 'EOF'
if false ; then echo hoge
elif false ; then
echo hoge
elif false ;then echo hoge
else
echo true
echo hoge
fi
EOF
)
[ "$res" = "true
hoge" ] || err $LINENO

res=$($com << 'EOF'
if false ;then echo hoge
else
echo true
echo hoge
fi
EOF
)
[ "$res" = "true
hoge" ] || err $LINENO

res=$($com << 'EOF'
if true ;then
echo true
echo hoge
fi
EOF
)
[ "$res" = "true
hoge" ] || err $LINENO

res=$($com << 'EOF'
if false ;then
echo a
elif true ;then
echo x
echo y
else 
echo true
echo hoge
fi
EOF
)
[ "$res" = "x
y" ] || err $LINENO

res=$($com << 'EOF'
if true ;then
    if true ;then
	echo a
    fi
fi
EOF
)
[ "$res" = "a" ] || err $LINENO

res=$($com << 'EOF'
if true ;then
    if true ;then
	echo a
	echo a
    fi
fi
EOF
)
[ "$res" = "a
a" ] || err $LINENO

res=$($com << 'EOF'
if true ;then
    if true ;then
	echo a
    fi
    echo a
fi
EOF
)
[ "$res" = "a
a" ] || err $LINENO

res=$($com << 'EOF'
if [ -f ~/.bash_aliases ]; then
    . ~/.bash_aliases
fi
EOF
)
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "b" ] ; then X=Y ; fi; echo $X')
[ "$res" = "Y" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "b" ] ; then X=Y ; fi | true; echo $X')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'if [ "a" == "a" ] ; then echo abcabc; elif [ "b" == "b" ] ; then X=Y ; fi > /tmp/tmp_x  ')
[ "$(cat $tmp )" = "abcabc" ]
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'if [ "$(cat)" == "abcabc" ] ; then echo xyz; elif [ "b" == "b" ] ; then X=Y ; fi < /tmp/tmp_x ')
[ "$res" = "xyz" ] || err $LINENO
res=$($com <<< 'if [ "$(cat)" == "xx" ] ; then echo xyz; elif [ "b" == "b" ] ; then echo pqr ; fi < /tmp/tmp_x ')
[ "$res" = "pqr" ] || err $LINENO
rm -f $tmp 

echo OK $0
