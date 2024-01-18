#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cargo build || err $LINENO

cd $(dirname $0)
com=../target/debug/sush

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ] || err $LINENO

### BUILTIN COMMAND TEST ###

res=$($com <<< 'cd /; pwd')
[ "$res" = "/" ] || err $LINENO

res=$($com <<< 'cd /tmp; mkdir hoge; ln -s hoge link; cd link; pwd -L; pwd -P')
[ "$res" = "/tmp/link
/tmp/hoge" ] ||
[ "$res" = "/tmp/link
/private/tmp/hoge" ] || err $LINENO

res=$($com <<< 'pwd -a 2>/tmp/rusty_bash; cat /tmp/rusty_bash')
[ "$res" = "sush: pwd: -a: invalid option
pwd: usage: pwd [-LP]" ] || err $LINENO

### COMPOUND COMMAND TEST ###

res=$($com <<< '(echo hoge; echo fuge)')
[ "$res" = "hoge
fuge" ] || err $LINENO

res=$($com <<< '(echo a; (echo b ; echo c) )')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '(
echo a; (echo b ; 
echo c) )')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '   (

echo a; (echo b ; 

echo c) )   ')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '   (#aaaa

echo a; (echo b ;  #bbb

echo c) )   ')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '(
echo a; (echo b ; 
')
[ "$?" = "2" ] || err $LINENO

res=$($com <<< '(echo hoge; false)')
[ "$?" = 1 ] || err $LINENO

res=$($com <<< 'cd / ; (cd /etc); pwd')
[ "$res" = / ] || err $LINENO

res=$($com <<< 'cd ; { cd / ; } ; pwd')
[ "$res" = / ] || err $LINENO

res=$($com <<< '( )')
[ "$?" = 2 ] || err $LINENO

res=$($com <<< '( echo a ; }')
[ "$?" = 2 ] || err $LINENO

res=$($com <<< '{ echo a ; }')
[ $res = "a" ] || err $LINENO

res=$($com <<< '{ echo a ; echo b ;}')
[ "$res" = "a
b" ] || err $LINENO

res=$($com <<< '{ echo a ; (echo b ; echo c) ;}')
[ "$res" = "a
b
c" ] || err $LINENO

res=$($com <<< '{ echo } ; }')
[ "$res" = "}" ] || err $LINENO

res=$($com <<< '{ echo a }')
[ "$?" = 2 ] || err $LINENO

### IRREGULAR COMMAND TEST ###

res=$($com <<< 'eeeeeecho hoge')
[ "$?" = 127 ] || err $LINENO

### PIPELINE ###

res=$($com <<< 'seq 10 | rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |
	rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |    

	  rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |  #コメントだよ

#コメントだよ
    #こめんとだよ

	  rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |   | head -n 1')
[ "$?" = "2" ] || err $LINENO

### COMMENT ###

res=$($com <<< 'echo a #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '
#comment comment
   #comment comment
echo a #aaaaa
#comment comment
')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a) #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a)#aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '{ echo a; }#aaaaa')
[ "$res" != "a" ] || err $LINENO

res=$($com <<< '{ echo a; } #aaaaa')
[ "$res" = "a" ] || err $LINENO

### NEW LINE ###

res=$($com <<< 'e\
c\
ho hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'e\
c\
ho \
hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge |\
rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< 'echo hoge |\
& rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< ' (seq 3; seq 3) | grep 3 | wc -l | tr -dc 0-9')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'ls |  | rev')
[ "$?" == "2" ] || err $LINENO

### REDIRECTS ###

# <, >, >>

res=$($com <<< 'cat < /etc/passwd | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'cat < /etc/passwd > /tmp/rusty_bash1 ; cat /tmp/rusty_bash1 | wc -l')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'echo a > /tmp/rusty_bash1 ; echo b >> /tmp/rusty_bash1; cat /tmp/rusty_bash1')
[ "$res" = "a
b" ] || err $LINENO

# non-fork redirects

res=$($com <<< '
	cd /etc/
	cd /tmp
	cd - > /tmp/rusty_bash1
	cd - > /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
/tmp" ] || err $LINENO

res=$($com <<< '
	cd /etc/
	cd /tmp
	{ cd - ; cd - ; /bin/echo a; } > /tmp/rusty_bash1
	{ cd - ; } > /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
a
/etc
/etc" ] || err $LINENO

# 2>, 2>>

res=$($com <<< '
	ls /aaaa 2> /tmp/rusty_bash
	ls /bbbb 2>> /tmp/rusty_bash
	cat /tmp/rusty_bash | grep ls | wc -l
	' | tr -dc 0-9)
[ "$res" = "2" ] || err $LINENO

# &>

res=$($com <<< 'ls /etc/passwd aaaa &> /tmp/rusty_bash; cat /tmp/rusty_bash | wc -l | tr -dc 0-9')
[ "$res" == "2" ] || err $LINENO

# &> for non-fork redirects

res=$($com <<< '
	{ ls /etc/passwd aaaa ; } &> /tmp/rusty_bash 
	cat /tmp/rusty_bash | wc -l | tr -dc 0-9')
[ "$res" == "2" ] || err $LINENO

res=$(LANG=C $com <<< '
	{ ls /etc/passwd aaaa ; } &> /tmp/rusty_bash 
	cat /tmp/rusty_bash | wc -l
	#ちゃんと標準出力が原状復帰されているか調査
	{ ls /etc/passwd ; }
	{ ls aaaa ; } 2> /tmp/rusty_bash2
	cat /tmp/rusty_bash2 | wc -l
	' | tr -d [:blank:])
[ "$res" == "2
/etc/passwd
1" ] || err $LINENO

res=$($com <<< '
	cd /etc/
	cd /tmp
	{ cd - ; cd - ; /bin/echo a; } &> /tmp/rusty_bash1
	{ cd - ; } &> /tmp/rusty_bash2
	cat /tmp/rusty_bash1
	cat /tmp/rusty_bash2
	pwd' | sed s@.private@@)
[ "$res" = "/etc
/tmp
a
/etc
/etc" ] || err $LINENO

### JOB PARSE TEST ###

res=$($com <<< '&& echo a')
[ "$?" == "2" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo a
&& echo b')
[ "$?" == "2" ] || err $LINENO

res=$($com <<< 'echo a &\
& echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&\
echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&
echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&



echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a ||
echo b')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'echo a \
&& echo b')
[ "$res" == "a
b" ] || err $LINENO

### WHILE TEST ###

res=$($com <<< 'touch /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "wait" ] || err $LINENO

res=$($com <<< 'rm -f /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'touch /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done > /tmp/rusty_bash1'; cat /tmp/rusty_bash1 ; cat /tmp/rusty_bash1 )
[ "$res" == "wait
wait" ] || err $LINENO

### IF TEST ###
res=$($com <<< 'if true ; then ; fi')
[ "$?" == "2" ] || err $LINENO

res=$($com <<< 'if ; then true ; fi')
[ "$?" == "2" ] || err $LINENO

res=$($com <<< 'if [ "a" == "a" ] ; then echo aa; fi')
[ "$res" = "aa" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; else echo bb; fi')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; fi' || echo x)
[ "$res" = "x" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo a ; fi ; if [ "b" == "b" ] ; then echo bb ; fi')
[ "$res" = "bb" ] || err $LINENO

res=$($com <<< 'echo file > /tmp/rusty_bash; if [ "a" == "a" ] ; then echo aa; fi >> /tmp/rusty_bash; cat /tmp/rusty_bash')
[ "$res" = "file
aa" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "c" ] ; then echo bb; else echo cc; fi')
[ "$res" = "cc" ] || err $LINENO

res=$($com <<< 'if [ "a" == "b" ] ; then echo aa; elif [ "b" == "c" ] ; then echo bb; elif [ "c" = "c" ] ; then echo cc ; else echo dd; fi')
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

echo OK $0
