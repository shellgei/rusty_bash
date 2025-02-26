#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

### COMPOUND COMMAND TEST ###
#
res=$($com -c '(echo a) aaaaaa')
[ "$?" = "2" ] || err $LINENO

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

### FUNCTION TEST ###

res=$($com <<< 'f () { echo a; } ; f')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< 'function f () { echo a; } ; f')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< 'function _f () { echo a; } ; _f')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< 'function f () { echo $A; } ; A=OK f')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'function f () { echo $A; } ; A=OK f | rev')
[ "$res" = "KO" ] || err $LINENO

res=$($com <<< 'function f () { A=BBB ; } ; f; echo $A')
[ "$res" = "BBB" ] || err $LINENO

res=$($com <<< 'function f () ( A=BBB ) ; f; echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'function f () { A=BBB ; } ; f | cat; echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'function f () { tr -d \\n ; } ; seq 3 | f')
[ "$res" = "123" ] || err $LINENO

res=$($com <<< 'set a b c ; function f () { echo $2 ; } ; f')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'set a b c ; function f () { echo $2 ; } ; f; echo $2')
[ "$res" = "
b" ] || err $LINENO

res=$($com <<< 'set a b c ; function f () { set 1 2 3 ; echo $2 ; } ; f; echo $2')
[ "$res" = "2
b" ] || err $LINENO

res=$($com <<< 'function f () { local A=BBB ; echo $A; } ; f ; echo $A')
[ "$res" = BBB ] || err $LINENO

res=$($com <<< 'A=3 ; function f () { local A ; A=BBB ; echo $A; } ; f ; echo $A')
[ "$res" = "BBB
3" ] || err $LINENO

res=$($com <<< 'function f () { local A=BBB ; echo $A ; } ; echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'function f () { local A=( a b c ) ; echo ${A[1]}; } ; f')
[ "$res" = b ] || err $LINENO

res=$($com <<< 'function f () { return; echo NG; } ; f')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'function f () { echo ok && return 3; } ; f')
[ "$?" = "3" ] || err $LINENO
[ "$res" = "ok" ] || err $LINENO

res=$($com <<< 'f () { g () { return; echo NG; } ; g ; echo OK; } ; f')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'f () { echo $#; } ; f x y z')
[ "$res" = "3" ] || err $LINENO

### WHILE TEST ###

res=$($com <<< 'touch /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "wait" ] || err $LINENO

res=$($com <<< 'rm -f /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'while false ; do echo do not come here ; done')
[ "$?" == 0 ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'touch /tmp/rusty_bash_x ; while [ -f /tmp/rusty_bash_x ] ; do echo wait ; rm /tmp/rusty_bash_x ; done > /tmp/rusty_bash_x1'; cat /tmp/rusty_bash_x1 ; cat /tmp/rusty_bash_x1 )
[ "$res" == "wait
wait" ] || err $LINENO

### FOR TEST ###

res=$($com <<< 'set a b c ; for x ; do echo $x ; done')
[ "$res" == "a
b
c" ] || err $LINENO

res=$($com <<< 'set a b c ; for x
    do echo $x
    done')
[ "$res" == "a
b
c" ] || err $LINENO

res=$($com <<< 'set a b c ; for x
do echo $x ; done')
[ "$res" == "a
b
c" ] || err $LINENO

res=$($com <<< 'for x in a b c ; do echo $x ; done')
[ "$res" == "a
b
c" ] || err $LINENO

res=$($com <<< 'for x in a{b,c} d ; do echo $x ; done')
[ "$res" == "ab
ac
d" ] || err $LINENO

res=$($com <<< 'set a b c ; for x in "$*" ; do echo $x ; done; for x in $* ; do echo $x ; done')
[ "$res" == "a b c
a
b
c" ] || err $LINENO

res=$($com <<< 'set a b c ; for x in "$@" ; do echo $x ; done')
[ "$res" == "a
b
c" ] || err $LINENO

res=$($com <<< 'set a b c ; for x in "@$@" ; do echo $x ; done')
[ "$res" == "@a
b
c" ] || err $LINENO

res=$($com <<< 'set a b c ; for x in "@$@x" ; do echo $x ; done')
[ "$res" == "@a
b
cx" ] || err $LINENO

res=$($com <<< 'A=(a b c); for x in "@${A[@]}x" ; do echo $x ; done')
[ "$res" == "@a
b
cx" ] || err $LINENO

res=$($com <<< 'A=(a b c); for x in "@${A[*]}x" ; do echo $x ; done')
[ "$res" == "@a b cx" ] || err $LINENO

res=$($com <<< 'set a; for x in "@$@x" ; do echo $x ; done')
[ "$res" == "@ax" ] || err $LINENO

res=$($com <<< 'set a b c ; for x in "@${@}x" ; do echo $x ; done')
[ "$res" == "@a
b
cx" ] || err $LINENO

res=$($com <<< 'for ((${ } ; ; )) ; do echo ; done')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'for ((i=0 ; i<2 ; i++ )) ; do echo a ; done')
[ "$res" == "a
a" ] || err $LINENO

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

res=$($com <<< 'function f () { echo a; if true ; then return ; fi ; echo b; } ; f')
[ "$res" = "a" ] || err $LINENO

res=$($com << 'EOF'
f()
{
	echo a;
}
EOF
)
[ "$?" = 0 ] || err $LINENO
[ "$res" = "" ] || err $LINENO


### CASE TEST ###

res=$($com <<< 'case aaa in bbb) echo NG ;; aaa) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in (bbb) echo NG ;; (aaa) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in bbb | aaa) echo OK ;; aaa) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in | aaa) echo OK ;; aaa) echo NG ;; esac')
[ "$?" = "2" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo NG ;; aaa) echo OK ;; aaa) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo NG ;; aaa) echo OK ;& aaa) echo OK ;; esac')
[ "$res" = "OK
OK" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo NG ;; aaa) echo OK ;;& aaa) echo OK ;; esac')
[ "$res" = "OK
OK" ] || err $LINENO

res=$($com <<< 'case aaa in
bbb) echo NG ;;
aaa) echo OK ;;&
aaa) echo OK ;;
	
	')
[ "$res" = "" ] || err $LINENO

res=$($com <<< '
case xterm-color in
    xterm-color|*-256color) color_prompt=yes;;
esac
echo $color_prompt
'
)
[ "$res" = "yes" ] || err $LINENO



res=$($com <<< 'case aaa in aaa) echo OK1 ;;& bbb) echo OK2 ;& aaa) echo OK3 ;; esac')
[ "$res" = "OK1
OK3" ] || err $LINENO

res=$($com <<< 'case aaa in aaa) echo OK1 ;& bbb) echo OK2 ;& aaa) echo OK3 ;; esac')
[ "$res" = "OK1
OK2
OK3" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo OK1 ;& bbb) echo OK2 ;& aaa) echo OK3 ;; esac')
[ "$res" = "OK3" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo OK1 ;& aaa) echo OK2 ;& aaa) echo OK3 ;; esac')
[ "$res" = "OK2
OK3" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo OK1 ;& aaa) echo OK2 ;& bbb) echo OK3 ;; esac')
[ "$res" = "OK2
OK3" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo OK1 ;& bbb) echo OK2 ;& bbb) echo OK3 ;; esac')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'echo ; case $? in 1) echo NG ;; 0) echo OK ;; esac')
[ "$res" = "
OK" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo NG ;; a\aa) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in bbb) echo NG ;; a\aa\ ) echo OK ;; esac')
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'case 山 in kawa) echo NG ;; 山) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in b*) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in ...) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case ... in aa) echo NG ;; ...) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case あ in ?) echo OK ;; あ) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case あbiuoあああ in ?) echo NG ;; あ*) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case yes in y[abcde]s) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case yes in y[abcde\]s) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case yes in y[^abcde]s) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case yes in y[^abcde]s) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case yes in y[\^abcde]s) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< "case yes in y'[e]'s) echo NG ;; *) echo OK ;; esac")
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case $- in *i*) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aaa in ?(a)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in ?(a)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case baa in ?(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case baa in @(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in @(a|b)aa) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in @(a||b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case acaa in @(a|b|ac)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case caa in !(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in !(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case "" in !(a|b)aa) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case cccccccccccaa in !(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case cccccccccccaa in *(a|b|c)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case "" in *(a|b|c)) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case "" in +(a|b|c)) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in *(a|b|c)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case cccccccccccaa in +(a|b|c)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case aa in +(a|b|c)aa) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 山田山田aa in +(山|山本|山田)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 山aa in 山*(本|田)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 山aa in 山+(本||田)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 山aa in 山*(本||田)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 山aa in 山+(本|田)aa) echo NG ;; *) echo OK ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 上山田山田aa in 上+(山|上山|山本|山田)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "OK" ] || err $LINENO

res=$($com <<< 'case 小山田 in !(五反|山|小山)田) echo マッチ ;; *) echo マッチせず ;; esac')
[ "$res" = "マッチせず" ] || err $LINENO

res=$($com <<< 'case 山小田 in !(山)田) echo マッチ ;; *) echo マッチせず ;; esac')
[ "$res" = "マッチ" ] || err $LINENO

res=$($com <<< 'case 山小小小田 in !(山)田) echo マッチ ;; *) echo マッチせず ;; esac')
[ "$res" = "マッチ" ] || err $LINENO

res=$($com <<< 'shopt -u extglob; case baa in @(a|b)aa) echo OK ;; *) echo NG ;; esac')
[ "$res" = "NG" ] || err $LINENO

res=$($com <<< 'shopt -u extglob; case 山小小小田 in !(山)田) echo マッチ ;; *) echo マッチせず ;; esac')
[ "$res" = "マッチせず" ] || err $LINENO

### (( )) TEST ###

res=$($com <<< '(( 0 ))')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< '(( 1 ))')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< '(( 0 + 1 + 2-3 ))')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< '(( 0 + 1 + 2+3 ))')
[ "$?" = "0" ] || err $LINENO

### [[ TEST ###

res=$($com -c '[[ -a /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -e /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -a /etc/passwdaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -e /etc/passwdaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ ! -e /dev/tty0 ]] || [[ -a /dev/tty0 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -a ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[ -a /etc/passwd x ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[ ! -a /etc/passwd ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ ! -a /etc/passwdaaa ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -a ( /etc/passwdaaa ) ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[ ! -a /dev/nvme0n1 ]] || [[ -b /dev/nvme0n1 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ ! -a /dev/tty0 ]] || [[ ! -b /dev/tty0 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ ! -a /dev/nvme0n1 ]] || [[ ! -c /dev/nvme0n1 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ ! -a /dev/tty0 ]] || [[ -c /dev/tty0 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -d /etc/ ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -d /etc/passwd ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -a ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[  ]]')
[ "$?" = "2" ] || err $LINENO

$com -c '[[ -f /dev/tty0 ]]'
[ "$?" = "1" ] || err $LINENO

if [ "$(uname)" = "Linux" ] ; then
	$com -c 'touch /tmp/$$ ; chmod g+s /tmp/$$; [[ -g /tmp/$$ ]] && rm /tmp/$$'
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -g /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -u /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c 'touch /tmp/$$ ; chmod u+s /tmp/$$; [[ -u /tmp/$$ ]] && rm /tmp/$$'
	[ "$?" = "0" ] || err $LINENO

	$com -c 'ln -s /etc/passwd /tmp/$$ ; [[ -h /tmp/$$ ]] && rm /tmp/$$'
	[ "$?" = "0" ] || err $LINENO

	$com -c 'ln -s /etc/passwd /tmp/$$ ; [[ -L /tmp/$$ ]] && rm /tmp/$$'
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -h /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -L /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -k /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -k /tmp/ ]]'
	[ "$?" = "0" ] || err $LINENO

	$com -c 'mkfifo /tmp/$$-fifo ; [[ -p /tmp/$$-fifo ]] && rm /tmp/$$-fifo '
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -p /tmp/ ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -r /etc/passwd ]]'
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -r /etc/shadow ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -x /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -x /bin/bash ]]'
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -x / ]]'
	[ "$?" = "0" ] || err $LINENO

	$com -c '[[ -S /bin/bash ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -S /dev/log ]]'
	[ "$?" = "0" ] || err $LINENO
fi

$com -c '[[ -s /etc/passwd ]]'
[ "$?" = "1" ] || err $LINENO

$com -c '[[ -s /etc/passwdaaaa ]]'
[ "$?" = "1" ] || err $LINENO

$com -c 'touch /tmp/$$-empty ; [[ -s /tmp/$$-empty ]]'
[ "$?" = "0" ] || err $LINENO

if [[ -t 1 ]] ; then
	$com -c '[[ -t 1 ]]'
	[ "$?" = "0" ] || err $LINENO
fi

echo | $com -c '[[ -t 0 ]]'
[ "$?" = "1" ] || err $LINENO

$com -c '[[ -t aaa ]]'
[ "$?" = "1" ] || err $LINENO

$com -c '[[ -w /etc/shadow ]]'
[ "$?" = "1" ] || err $LINENO

$com -c '[[ -w /etc ]]'
[ "$?" = "1" ] || err $LINENO

$com -c 'touch /tmp/$$-file; [[ -w /tmp/$$-file ]]; rm /tmp/$$-file'
[ "$?" = "0" ] || err $LINENO

$com -c 'touch /tmp/$$-file; [[ -w /tmp/$$-file ]]; rm /tmp/$$-file'
[ "$?" = "0" ] || err $LINENO

$com -c '[[ -G ~ ]]'
[ "$?" = "0" ] || err $LINENO

$com -c '[[ -O ~ ]]'
[ "$?" = "0" ] || err $LINENO

if [ "$(whoami)" != root ] ; then
	$com -c '[[ -G /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO

	$com -c '[[ -O /etc/passwd ]]'
	[ "$?" = "1" ] || err $LINENO
fi

res=$($com -c '
touch /tmp/$$-N
[[ -N /tmp/$$-N ]] ; echo $?
echo a >> /tmp/$$-N
[[ -N /tmp/$$-N ]] ; echo $?
[[ -N /tmp/$$-N ]] ; echo $?
cat /tmp/$$-N > /dev/null
[[ -N /tmp/$$-N ]] ; echo $?
rm /tmp/$$-N')
[ "$res" = "1
0
0
1" ] || err $LINENO

res=$($com -c '[[ (-a /etc/passwd) ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ ( ! -a /etc/passwd ) ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ ! ( -a /etc/passwd ) ]]')
[ "$?" = "1" ] || err $LINENO

# file compare

rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; ln /tmp/$$ /tmp/$$x; [[ /tmp/$$ -ef /tmp/$$x ]]')
[ "$?" = "0" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; touch /tmp/$$x; [[ /tmp/$$ -ef /tmp/$$x ]]')
[ "$?" = "1" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; touch /tmp/$$x; [[ ! /tmp/$$ -ef /tmp/$$x ]]')
[ "$?" = "0" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; touch /tmp/$$x; [[ ! ( /tmp/$$ -ef /tmp/$$x ) ]]')
[ "$?" = "0" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c '[[ /etc/passwd -ef /tmp/aaaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ /etc/aaaaaa -ef /etc/passwd ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'touch /tmp/$$ ; sleep 0.01 ; touch /tmp/$$x; [[ /tmp/$$x -nt /tmp/$$ ]]')
[ "$?" = "0" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; sleep 0.01 ; touch /tmp/$$x; [[ /tmp/$$ -nt /tmp/$$x ]]')
[ "$?" = "1" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; [[ /tmp/$$ -nt /tmp/$$ ]]')
[ "$?" = "1" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c '[[ /etc/passwd -nt /tmp/aaaaaaaaa ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ /etc/aaaaaaaaaa -nt /etc/bbbbbb ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'touch /tmp/$$ ; sleep 0.01; touch /tmp/$$x; [[ /tmp/$$x -ot /tmp/$$ ]]')
[ "$?" = "1" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; sleep 0.01; touch /tmp/$$x; [[ /tmp/$$ -ot /tmp/$$x ]]')
[ "$?" = "0" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c 'touch /tmp/$$ ; [[ /tmp/$$ -ot /tmp/$$ ]]')
[ "$?" = "1" ] || err $LINENO
rm -f /tmp/$$*

res=$($com -c '[[ /etc/passwd -ot /tmp/aaaaaaaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ /etc/aaaaaaaaaa -ot /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ /etc/aaaaaaaaaa -ot /etc/bbbbbb ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -ot ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -o pipefail ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'set -o pipefail ; [[ -o pipefail ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -o extglob ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -o pipefailaaaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -v LANG ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -v LANGLANG ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A= ; [[ -v A ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'B=A; A= ; [[ -v $B ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -v "$B" ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -z "" ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -z ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[ -z a ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -z "$BASH_VERSION" ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -z "$aaaa" ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ "aaaa" ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ "" ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -n "aaaa" ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -n "" ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ あいう = $A ]]')
[ "$?" = "0" ] || err $LINENO
res=$($com -c 'A=あいう ; [[ あいう == $A ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいうえ ; [[ あいう = $A ]]')
[ "$?" = "1" ] || err $LINENO
res=$($com -c 'A=あいうえ ; [[ あいう == $A ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A=あいうえ ; [[ あいう != $A ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ あいう != $A ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ != $A ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c '[[ aaa != ]]')
[ "$?" = "2" ] || err $LINENO

$com -c '[[  ==  ]] && [[ = ]] && [[ != ]]'
[ "$?" = "0" ] || err $LINENO

res=$($com <<< '[[ ! ]]')
[ $? -eq 2 ] || err $LINENO

res=$($com <<< '[[ ! $a ]]')
[ $? -eq 0 ] || err $LINENO

res=$($com <<< 'a=1 ; [[ ! $a ]]')
[ $? -eq 1 ] || err $LINENO

$com -c '[[ abc > aaa ]] && [[ 0100 < 2 ]] && [[ ! abc > abc ]]'
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ $A =~ あ ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう; RE="あ*" ; [[ $A =~ $RE ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう; RE="あ*" ; [[ $A =~ ${RE}う ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう; RE="あ*" ; [[ $A =~ ${RE}お ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ $A =~ ... ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ A =~ あ ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ $A =~ * ]]')
[ "$?" = "2" ] || err $LINENO

res=$($com -c 'A=あいう ; [[ $A =~ (.)(..) ]]; echo ${BASH_REMATCH[@]}')
[ "$?" = "0" ] || err $LINENO
[ "$res" = "あいう あ いう" ] || err $LINENO

# and or 

res=$($com -c '[[ -a /etc/passwd && -a /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -a /etc/passwd &&
-a /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ -a /etc/passwd && -a /etc/passwdaaa ]]')
[ "$?" = "1" ] || err $LINENO

res=$($com -c '[[ -a /etc/passwdaaaa || -a /etc/passwd ]]')
[ "$?" = "0" ] || err $LINENO

# glob

res=$($com -c '[[ $- == Bc ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ $- == *c* ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ $- != *c* ]]')
[ "$?" = "1" ] || err $LINENO

# calculation

res=$($com -c '[[ 3 -eq 1+2 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ 1+2 -eq 3 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ "1 + 2" -eq 3 ]]')
[ "$?" = "0" ] || err $LINENO

res=$($com -c '[[ "10#1 + 2" -eq 3 ]]')
[ "$?" = "0" ] || err $LINENO

echo $0 >> ./ok
