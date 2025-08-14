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

res=$($com <<< 'echo (')
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

#res=$($com <<< 'e\
#c\
#ho hoge')
#[ "$res" = "hoge" ] || err $LINENO
#
#res=$($com <<< 'e\
#c\
#ho \
#hoge')
#[ "$res" = "hoge" ] || err $LINENO
#
#res=$($com <<< 'echo hoge |\
#rev')
#[ "$res" = "egoh" ] || err $LINENO
#
#res=$($com <<< 'echo hoge |\
#& rev')
#[ "$res" = "egoh" ] || err $LINENO
#
#res=$($com <<< ' (seq 3; seq 3) | grep 3 | wc -l | tr -dc 0-9')
#[ "$res" = "2" ] || err $LINENO
#
#res=$($com <<< 'ls |  | rev')
#[ "$?" == "2" ] || err $LINENO

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

# with expansion

res=$($com <<< 'echo a > {a,b}' 2>&1)
[ "$?" == "1" ] || err $LINENO
[ "$res" == "sush: {a,b}: ambiguous redirect" ] || err $LINENO


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

### ARG TEST ###

# brace

res=$($com <<< 'echo {a,b}c')
[ "$res" == "ac bc" ] || err $LINENO

res=$($com <<< 'echo c{a,b}')
[ "$res" == "ca cb" ] || err $LINENO

res=$($com <<< 'echo {{a},b}')
[ "$res" == "{a} b" ] || err $LINENO

res=$($com <<< 'echo {a,{b},c}')
[ "$res" == "a {b} c" ] || err $LINENO

res=$($com <<< 'echo {a,b,c{d,e}f,g{h,i{j,k}}}')
[ "$res" == "a b cdf cef gh gij gik" ] || err $LINENO

res=$($com <<< 'echo {a,b,c{d,e}f,g{h,i{j,k}}')
[ "$res" == "{a,b,cdf,gh {a,b,cdf,gij {a,b,cdf,gik {a,b,cef,gh {a,b,cef,gij {a,b,cef,gik" ] || err $LINENO

res=$($com <<< 'echo c{a,b')
[ "$res" == "c{a,b" ] || err $LINENO

res=$($com <<< 'echo c{a,b,')
[ "$res" == "c{a,b," ] || err $LINENO

res=$($com <<< 'echo {{a,あいうえお@},{c,d},')
[ "$res" == "{a,c, {a,d, {あいうえお@,c, {あいうえお@,d," ] || err $LINENO

res=$($com <<< 'echo {{a,b},{c,d')
[ "$res" == "{a,{c,d {b,{c,d" ] || err $LINENO

res=$($com <<< 'echo {{a,b,{c,')
[ "$res" == "{{a,b,{c," ] || err $LINENO

res=$($com <<< 'echo {a}')
[ "$res" == "{a}" ] || err $LINENO

res=$($com <<< 'echo {a,}')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'echo {a,b,}')
[ "$res" == "a b" ] || err $LINENO

res=$($com <<< 'echo {a,b,}c')
[ "$res" == "ac bc c" ] || err $LINENO

res=$($com <<< 'echo {}')
[ "$res" == "{}" ] || err $LINENO

res=$($com <<< 'echo {,}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo {,,}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo a{,,}b')
[ "$res" == "ab ab ab" ] || err $LINENO

res=$($com <<< 'echo {')
[ "$res" == "{" ] || err $LINENO

res=$($com <<< 'echo }')
[ "$res" == "}" ] || err $LINENO

res=$($com <<< 'echo {a,}{b,}')
[ "$res" == "ab a b" ] || err $LINENO

res=$($com <<< 'echo {d}d{},dba}')
[ "$res" == "d}d{} dba" ] || err $LINENO
#[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo {}a,b}')
[ "$res" == "{}a,b}" ] || err $LINENO

res=$($com <<< 'echo c{}a,b}')
[ "$res" == "c}a cb" ] || err $LINENO

res=$($com <<< 'echo {,}{}a,b}')
[ "$res" == "{}a,b} {}a,b}" ] || err $LINENO

res=$($com <<< 'echo a{}},b}')
[ "$res" == "a}} ab" ] || err $LINENO

res=$($com <<< 'echo $${a,b}')
[ "$res" != "\$\${a,b}" ] || err $LINENO

res=$($com <<< 'echo $${a,{b,c},d}')
[ "$res" != "\$\${a,{b,c},d}" ] || err $LINENO

res=$($com <<< 'echo あ{a,b}{},c}')
[ "$res" == "あa{},c} あb{},c}" ] || err $LINENO

res=$($com <<< 'echo あ{a,b}d{},c}')
[ "$res" == "あad} あadc あbd} あbdc" ] || err $LINENO

# escaping

res=$($com <<< "echo a\ \ \ a")
[ "$res" == "a   a" ] || err $LINENO

res=$($com <<< 'echo \(')
[ "$res" == "(" ] || err $LINENO

# quotation

res=$($com <<< "echo 'abc'")
[ "$res" == "abc" ] || err $LINENO

res=$($com <<< "echo 'abあいうc'")
[ "$res" == "abあいうc" ] || err $LINENO

res=$($com <<< "echo 123'abc'")
[ "$res" == "123abc" ] || err $LINENO

res=$($com <<< "echo 123'abc'def")
[ "$res" == "123abcdef" ] || err $LINENO


### WHILE TEST ###

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

### word split

export SUSH_TEST="ab
cd
ef"
res=$($com <<< 'echo @$SUSH_TEST@')
[ "$res" = "@ab cd ef@" ] || err $LINENO

export SUSH_TEST=" ab
cd
ef
"
res=$($com <<< 'echo @$SUSH_TEST@')
[ "$res" = "@ ab cd ef @" ] || err $LINENO

### glob

res=$($com <<< 'echo /bin/?' | grep -F '/bin/[')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo /*' | grep '/etc')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo ~+/*' | grep '*')
[ "$?" == 1 ] || err $LINENO

#res=$($com <<< 'echo ~/*' | grep -F '/.')
#[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'echo ~/.*' | grep -F '/.')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo /etc*/' | grep -F '/etc/')
[ "$?" == 0 ] || err $LINENO

res=$($com <<< 'echo .*' | grep -F './.')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'echo ./*' | grep -F './')
[ "$?" == "0" ] || err $LINENO

#res=$($com <<< 'echo *"$PATH"')
#[ "$?" == "0" ] || err $LINENO
#
#res=$($com <<< 'echo /*"b"*' | grep -F '*')
#[ "$?" == "1" ] || err $LINENO

res=$($com <<< "echo /*'b'*" | grep -F '*')
[ "$?" == "1" ] || err $LINENO

#res=$($com <<< 'echo /"*"' | grep -F '*')
#[ "$?" == "0" ] || err $LINENO

### DOUBLE QUOTATION ###

res=$($com <<< 'echo "*"')
[ "$res" == "*" ] || err $LINENO

res=$($com <<< 'echo "{a,{b},c}"')
[ "$res" == "{a,{b},c}" ] || err $LINENO

#export RUSTY_BASH_A='a
#b'
#res=$($com <<< 'echo "$RUSTY_BASH_A"')
#[ "$res" == "a
#b" ] || err $LINENO

#res=$($com <<< 'echo "$BASH{PID,_SUBSHELL}"')
#[ "$res" == "{PID,_SUBSHELL}" ] || err $LINENO

#res=$($com <<< 'echo "\$HOME"')
#[ "$res" == '$HOME' ] || err $LINENO

res=$($com <<< 'echo "\a"')
[ "$res" == '\a' ] || err $LINENO

#res=$($com <<< 'echo "\\"')
#[ "$res" == '\' ] || err $LINENO

res=$($com <<< 'echo "a   b"')
[ "$res" == 'a   b' ] || err $LINENO

res=$($com <<< 'echo "a
b
c"')
[ "$res" == 'a
b
c' ] || err $LINENO

res=$($com <<< 'echo "')
[ "$?" == 2 ] || err $LINENO

res=$($com <<< 'echo "" a')
[ "$res" == " a" ] || err $LINENO

#res=$($com <<< 'set a b c; echo a"$@"c')
#[ "$res" == "aa b cc" ] || err $LINENO

#res=$($com <<< 'set a b c; echo $#')
#[ "$res" == "3" ] || err $LINENO

#res=$($com <<< 'set a b c; A=( A"$@"C ); echo ${A[0]}')
#[ "$res" == "Aa" ] || err $LINENO
#
#res=$($com <<< 'set a b c; A=( A"$@"C ); echo ${A[2]}')
#[ "$res" == "cC" ] || err $LINENO
#
#res=$($com <<< 'set a b c; A=( A"$*"C ); echo ${A[0]}')
#[ "$res" == "Aa b cC" ] || err $LINENO
#
#res=$($com <<< 'set a b c; A=( A$*C ); echo ${A[1]}')
#[ "$res" == "b" ] || err $LINENO
#
#res=$($com <<< 'set a; A=( A"$@"C ); echo ${A[0]}')
#[ "$res" == "AaC" ] || err $LINENO
#
#res=$($com <<< 'A=( A"$@"C ); echo ${A[0]}')
#[ "$res" == "AC" ] || err $LINENO
#
#res=$($com <<< 'set あ; echo a"$@"c')
#[ "$res" == "aあc" ] || err $LINENO
#
#res=$($com <<< 'set あ い; echo a"$@"c')
#[ "$res" == "aあ いc" ] || err $LINENO

#res=$($com <<< 'echo a"$@"c')
#[ "$res" == "ac" ] || err $LINENO

### return ###

res=$($com <<< 'function f () { return; echo NG; } ; f')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'function f () { echo ok && return 3; } ; f')
[ "$?" = "3" ] || err $LINENO
[ "$res" = "ok" ] || err $LINENO

res=$($com <<< 'f () { g () { return; echo NG; } ; g ; echo OK; } ; f')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< '
f () {
        g () {
                h () {
                        return
                        echo NG
                }
                h
                echo OK
        }
        g
        echo OK
        return
        echo NG
}
f
')
[ "$res" = "OK
OK" ] || err $LINENO

res=$($com <<< 'function f () { echo a; if true ; then return ; fi ; echo b; } ; f')
[ "$res" = "a" ] || err $LINENO

### substitution ###

res=$($com <<< 'A=/*; echo $A | grep -q "*"')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=/*; echo $A | grep -q "etc"')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'A=BBB; echo $A')
[ "$res" == "BBB" ] || err $LINENO

res=$($com <<< 'A=BBB echo ok')
[ "$res" == "ok" ] || err $LINENO

res=$($com <<< 'A=BBB B= echo ok')
[ "$res" == "ok" ] || err $LINENO

res=$($com <<< 'A=A$(echo BBB)C; echo $A')
[ "$res" == "ABBBC" ] || err $LINENO

res=$($com <<< 'A={a,b}; echo $A')
[ "$res" == "{a,b}" ] || err $LINENO

### local substitution ###

#res=$($com <<< 'A=ABC cd; echo A')
#[ "$res" == "" ] || err $LINENO

echo OK $0
