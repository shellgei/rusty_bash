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

### POSITIONAL PARAMETERS ###

cat << 'EOF' > $tmp 
echo $1 $2 $3
EOF

res=$(cat $tmp  | $com a b c)
[ "$res" = "a b c" ] || err $LINENO

#### ARG TEST ###

res=$($com <<< 'echo aaa"bbb"')
[ "$res" = "aaabbb" ] || err $LINENO

res=$($com << 'EOF'
echo 'a' "b  b" cc  c
EOF
)
[ "$res" = "a b  b cc c" ] || err $LINENO

res=$($com << 'EOF'
echo "\"" "\\" a\ \ bc
EOF
)
[ "$res" = '" \ a  bc' ] || err $LINENO

res=$($com <<< 'echo "\a\n\$\`\{\}"')
[ "$res" = '\a\n$`{}' ] || err $LINENO

res=$($com << 'EOF'
echo "a'b'c"
EOF
)
[ "$res" = "a'b'c" ] || err $LINENO

res=$($com <<< 'echo hoge"hoge";')
[ "$res" = "hogehoge" ] || err $LINENO

res=$($com <<< "echo '\*'")
[ "$res" = "\*" ] || err $LINENO

res=$($com <<< 'echo )' || true)
[ "$res" = "" ] || err $LINENO

# brace expansion

res=$($com << 'EOF'
echo {a}
EOF
)
[ "$res" = '{a}' ] || err $LINENO

res=$($com << 'EOF'
echo {a,b}{cc,dd}
EOF
)
[ "$res" = 'acc add bcc bdd' ] || err $LINENO

res=$($com << 'EOF'
echo "{a,b}{cc,dd}"
EOF
)
[ "$res" = '{a,b}{cc,dd}' ] || err $LINENO

res=$($com << 'EOF'
echo あ{cc,いうえお}
EOF
)
[ "$res" = 'あcc あいうえお' ] || err $LINENO

res=$($com << 'EOF'
echo {a,b}{c,d}へ{e,f}
EOF
)
[ "$res" = 'acへe acへf adへe adへf bcへe bcへf bdへe bdへf' ] || err $LINENO

res=$($com << 'EOF'
echo {,b,c}{a,b}
EOF
)
[ "$res" = 'a b ba bb ca cb' ] || err $LINENO

res=$($com << 'EOF'
echo {a,"b,c",'d,e',f}
EOF
)
[ "$res" = 'a b,c d,e f' ] || err $LINENO

res=$($com <<< 'echo {a,b{c,d},e}')
[ "$res" = "a bc bd e" ] || err $LINENO

res=$($com <<< 'echo {a,*}zzzzz')
[ "$res" = "azzzzz *zzzzz" ] || err $LINENO

res=$($com <<< 'echo {')
[ "$res" = '{' ] || err $LINENO

res=$($com <<< 'echo a{b{c')
[ "$res" = 'a{b{c' ] || err $LINENO

res=$($com <<< 'echo a{b{c}')
[ "$res" = 'a{b{c}' ] || err $LINENO

res=$($com <<< 'echo a{b{}')
[ "$res" = 'a{b{}' ] || err $LINENO

res=$($com <<< 'echo }')
[ "$res" = '}' ] || err $LINENO

# glob test

res=$($com << 'EOF'
ls t*t.bash
EOF
)
[ "$res" = "test.bash" ] || err $LINENO

res=$($com <<< 'echo "*"')
[ "$res" = "*" ] || err $LINENO

res=$($com <<< 'echo /')
[ "$res" = "/" ] || err $LINENO

#The following checks trivial difference between bash and this.
#$com <<< 'echo //*' | grep -F '//' 
#$com <<< 'echo /*////' | grep -Fv '//'

# command substitution

res=$($com <<< 'echo $(echo hoge)hoge')
[ "$res" = "hogehoge" ] || err $LINENO

res=$($com <<< 'echo hoge$(echo hoge)')
[ "$res" = "hogehoge" ] || err $LINENO

res=$($com <<< 'echo "hoge$(echo hoge)"')
[ "$res" = "hogehoge" ] || err $LINENO

res=$($com <<< 'echo "hoge$(echo hoge; echo hoge)"')
[ "$res" = "hogehoge
hoge" ] || err $LINENO

res=$($com <<< 'echo "$(seq 2)"')
[ "$res" = "1
2" ] || err $LINENO

res=$($com <<< 'echo a"$(seq 4)"b')
[ "$res" = "a1
2
3
4b" ] || err $LINENO

res=$($com <<< 'echo "$(seq 3)"{a,b}')
[ "$res" = "1
2
3a 1
2
3b" ] || err $LINENO

res=$($com <<< 'echo {a,"$(seq 3)"}b')
[ "$res" = "ab 1
2
3b" ] || err $LINENO

res=$($com <<< 'echo {a,$(seq 2)"$(seq 2)"{$(seq 2),"$(seq 2)"}}')
[ "$res" = "a 1 21
21 2 1 21
21
2" ] || err $LINENO

res=$($com <<< 'echo $(seq 3){a,b}')
[ "$res" = "1 2 3a 1 2 3b" ] || err $LINENO

res=$($com <<< 'cd /;echo "$(pwd)x"') #internal command
[ "$res" = "/x" ] || err $LINENO

$com <<< 'ls $(echo / /)'
[ $? -eq 0 ] || err $LINENO

res=$($com <<< 'echo {$(seq 5)}')
[ "$res" = "{1 2 3 4 5}" ] || err $LINENO

res=$($com <<< 'echo {$(seq 3),$(seq 3)')
[ "$res" = "{1 2 3,1 2 3" ] || err $LINENO


res=$($com <<< 'echo $( echo abc | (rev) )') 
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'echo $( echo a ;  ( echo b ; echo c ) )')
[ "$res" = "a b c" ] || err $LINENO

res=$($com <<< 'echo a |  { cat ; exit 3 ; }; echo $?')
[ "$res" = "a
3" ] || err $LINENO

res=$($com <<< 'A=$(seq 2);echo $A; echo "$A"')
[ "$res" = "1 2
1
2" ] || err $LINENO

res=$($com <<< 'A=$(seq 2 | sed "s-^- -"); echo "$A"' )
[ "$res" = " 1
 2" ] || err $LINENO

res=$($com <<< 'A="
1
2
3"; echo "$A"' )
[ "$res" = "
1
2
3" ] || err $LINENO

res=$($com << 'EOF'
A='
1
 2
  3'

echo "$A"
EOF
)
[ "$res" = '
1
 2
  3' ] || err $LINENO

# expansion of tilde

res=$($com <<< 'echo ~')
[ "$res" = "$HOME" ] || err $LINENO

res=$($com <<< 'echo "~"')
[ "$res" = "~" ] || err $LINENO

res=$($com <<< 'echo ~/')
[ "$res" = "$HOME/" ] || err $LINENO

res=$($com <<< 'echo ~a')
[ "$res" = "~a" ] || err $LINENO

res=$($com <<< 'echo ~*')
[ "$res" = "~*" ] || err $LINENO

user=$(tail -n 1 /etc/passwd | awk -F: '{print $1}')
home=$(tail -n 1 /etc/passwd | awk -F: '{print $(NF-1)}')

res=$($com <<< "echo ~$user")
[ "$res" = "$home" ] || err $LINENO

res=$($com <<< "echo {~$user,a}")
[ "$res" = "$home a" ] || err $LINENO

### DIRECTORY TEST ###

res=$($com << 'EOF'
cd /
pwd
EOF
)
[ "$res" = "/" ] || err $LINENO

### COMMAND BOUNDARY TEST ###

res=$($com <<< 'echo hoge;echo hoge')
[ "$res" = "hoge
hoge" ] || err $LINENO

### COMMENT TEST ###

res=$($com << 'EOF'
echo hello #HEHEHEHEHEあいうえお
echo world
EOF
)
[ "$res" = 'hello
world' ] || err $LINENO

### VARIABLE TEST ###

res=$($com << 'EOF'
abc=あいうえお
echo $abc
echo ${abc}
echo "a${abc}'b'c"
abc=
echo $abc
EOF
)
[ "$res" = "あいうえお
あいうえお
aあいうえお'b'c" ] || err $LINENO

res=$($com <<< 'a={a,b}{c,d};echo $a')
[ "$res" = "{a,b}{c,d}" ] || err $LINENO

res=$($com << 'EOF'
abc=あいうえお
def=${abc}かきくけこ
echo $def
EOF
)
[ "$res" = "あいうえおかきくけこ" ] || err $LINENO

res=$($com <<< 'echo ${a:-b}')
[ "$res" = "b" ] || err $LINENO

res=$($com <<< 'a=c;echo ${a:-b}')
[ "$res" = "c" ] || err $LINENO

res=$($com <<< 'echo ${a:=b};echo $a')
[ "$res" = "b
b" ] || err $LINENO

res=$($com <<< 'echo ${a:-b};echo $a')
[ "$res" = "b" ] || err $LINENO

res=$($com <<< 'a=c;echo ${a:?b}')
[ "$res" = "c" ] || err $LINENO

res=$($com <<< '(echo ${a:?b}) 2>&1')
[ "$res" = "bash: a: b" ] || err $LINENO

res=$($com <<< 'LANG=C TZ= date -d 2000-01-01')
[ "$res" = "Sat Jan  1 00:00:00 UTC 2000" ] || err $LINENO

res=$($com << 'EOF'
LANG=C
TZ= date -d 2000-01-01
EOF
)
[ "$res" = "Sat Jan  1 00:00:00 UTC 2000" ] || err $LINENO

# special variable

res=$($com <<< 'ls aaaaaaa; echo $?')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'echo $$')
[ "$res" -gt 1 ] || err $LINENO

cat << 'EOF' > $tmp 
echo $@
echo $*
IFS=💩
echo "$*"
EOF

res=$($com -x <<< 'echo $-')
[ "$res" = "x" ] || err $LINENO

res=$(cat $tmp  | $com あい うえ お)
[ "$res" = "あい うえ お
あい うえ お
あい💩うえ💩お" ] || err $LINENO

res=$($com <<< 'A=x;echo a $A; echo $_')
[ "$res" = "a x
x" ] || err $LINENO

### REDIRECTION ###

res=$($com << 'EOF'
echo text > /tmp/tmp_x
wc < /tmp/tmp_x
rm /tmp/tmp_x
EOF
)
[ "$res" = "1 1 5" ] || err $LINENO

res=$($com << 'EOF'
echo text 1> /tmp/tmp_x
wc 0< /tmp/tmp_x
rm /tmp/tmp_x
EOF
)
[ "$res" = "1 1 5" ] || err $LINENO

$com << 'EOF' | grep 'aaaa'
ls aaaaaaaaaaaaaaaaaaaaaa 2> /tmp/tmp_x 
cat /tmp/tmp_x 
rm /tmp/tmp_x
EOF

res=$($com << 'EOF' 
ls -d / aaaaaaaaaaaaaaaaaaaa &> /tmp/tmp_x 
wc -l < /tmp/tmp_x 
rm /tmp/tmp_x 
EOF
)
[ "$res" = "2" ] || err $LINENO

res=$($com << 'EOF' 
ls -d /hogehgoe 2>&1
EOF
)
[ "$(echo $res | wc -l)" = "1" ] || err $LINENO

res=$($com << 'EOF' 
echo $(ls /aaaa 2>&1)x
EOF
)
echo "$res" | grep x | grep ls || err $LINENO

res=$($com << 'EOF'
ls aaaaaaaaaaaaaaaaaaa > /tmp/tmp_x  2>&1
wc -l < /tmp/tmp_x
rm /tmp/tmp_x 
EOF
)

[ "$res" = "1" ] || err $LINENO

res=$($com << 'EOF'
2>/tmp/tmp_x  echo hoge
rm /tmp/tmp_x
EOF
)
[ "$res" = "hoge" ] || err $LINENO

res=$($com << 'EOF'
echo 2>/tmp/tmp_x  hoge
rm /tmp/tmp_x 
EOF
)
[ "$res" = "hoge" ] || err $LINENO

res=$($com << 'EOF'
A=B >/tmp/tmp_x  C=D echo hoge
cat /tmp/tmp_x 
rm /tmp/tmp_x
EOF
)
[ "$res" = "hoge" ] || err $LINENO

res=$($com << 'EOF' 
ls -d /hogehgoe 2>&$(echo 1) | wc -l
EOF
)
[ "$res" = "1" ] || err $LINENO

res=$($com << 'EOF' 
ls -d /hogehgoe 2>&$(echo 1 2)
EOF
)
[ "$?" = "1" ] || err $LINENO

### PIPELINE ###

res=$($com <<< 'echo abc | rev')
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'echo abc | rev | tr abc def')
[ "$res" = "fed" ] || err $LINENO

res=$($com <<< '! echo abc | rev | tr abc def')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< '! echo abc | rev | false')
[ "$?" = "0" ] || err $LINENO

### JOB ###

res=$($com <<< '(sleep 1; echo a) & echo b')
[ "$res" = "b
a" ] || err $LINENO

res=$($com <<< '(sleep 1; echo a) & wait ; echo b')
[ "$res" = "a
b" ] || err $LINENO

### COMPOUND COMMAND ###

res=$($com <<< '(echo hoge)')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '{echo hoge; }')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '(echo hoge;echo hoge)')
[ "$res" = "hoge
hoge" ] || err $LINENO

res=$($com <<< '{echo hoge;echo hoge ; }')
[ "$res" = "hoge
hoge" ] || err $LINENO

res=$($com <<< '(echo hoge | rev;echo hoge)')
[ "$res" = "egoh
hoge" ] || err $LINENO

res=$($com <<< 'echo abc | ( echo a ; rev ) | tr -d \\n')
[ "$res" = "acba" ] || err $LINENO

res=$($com <<< '{echo hoge | rev;echo hoge ; }')
[ "$res" = "egoh
hoge" ] || err $LINENO

res=$($com <<< 'echo abc | { echo a ; rev ; } | tr -d \\n')
[ "$res" = "acba" ] || err $LINENO

res=$($com <<< '(A=B);echo $A')
[ "$res" = "" ] || err $LINENO

res=$($com <<< '{A=B ; };echo $A')
[ "$res" = "B" ] || err $LINENO

res=$($com <<< 'echo abc | (rev)')
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< '(echo abc) | rev')
[ "$res" = "cba" ] || err $LINENO

res=$($com << 'EOF'
(ls aaaaa) 2> /tmp/tmp_x
cat /tmp/tmp_x  | wc -l
rm /tmp/tmp_x 
EOF
)
[ "$res" -ge 1 ] || err $LINENO

res=$($com <<< '{ echo } ; }')
[ "$res" = "}" ] || err $LINENO

res=$($com <<< '((echo hoge) )')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '((echo hoge))')
[ "$?" != "0" ] || err $LINENO
[ "$res" = "" ] || err $LINENO

# compound and read

res=$($com <<< 'echo あ い う | ( read b ; echo $b )')
[ "$res" = "あ い う" ] || err $LINENO

res=$($com <<< 'echo あ い う | ( read a b ; echo $b )')
[ "$res" = "い う" ] || err $LINENO

res=$($com <<< 'echo あ い う | ( read a b c ; echo $b )')
[ "$res" = "い" ] || err $LINENO

# (())

res=$($com <<< '((0));echo $?')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< '((1));echo $?')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'echo $((1+2+3))')
[ "$res" = "6" ] || err $LINENO

res=$($com <<< 'echo $((1-2+3))')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'echo $((1+2*3))')
[ "$res" = "7" ] || err $LINENO

res=$($com <<< 'echo $((1+2*-3))')
[ "$res" = "-5" ] || err $LINENO

res=$($com <<< 'echo $((1+2/3))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'echo $((-1+2/3))')
[ "$res" = "-1" ] || err $LINENO

### MULTILINE INPUT ###

res=$($com << 'EOF'
echo a \
b \
c
EOF
)
[ "$res" = "a b c" ] || err $LINENO

res=$($com << 'EOF'
ec\
ho a\
b\
c
EOF
)
[ "$res" = "abc" ] || err $LINENO

res=$($com << 'EOF'
(
echo a
echo b
)
EOF
)
[ "$res" = "a
b" ] || err $LINENO

res=$($com << 'EOF'
{
echo a
echo b
}
EOF
)
[ "$res" = "a
b" ] || err $LINENO

res=$($com << 'EOF'
echo abc |
rev
EOF
)
[ "$res" = "cba" ] || err $LINENO

### FUNCTION ###

res=$($com << 'EOF'
somefunc () {
	echo a
}

somefunc
EOF
)
[ "$res" = "a" ] || err $LINENO

res=$($com << 'EOF'
function somefunc () {
	echo a
}

somefunc
somefunc
EOF
)
[ "$res" = "a
a" ] || err $LINENO

res=$($com << 'EOF'
somefunc (    ) {
	echo abc
}

somefunc | rev
EOF
)
[ "$res" = "cba" ] || err $LINENO

res=$($com << 'EOF'
somefunc () {
	echo a
	rev
}

echo abc | somefunc | tr -d '\n'
EOF
)
[ "$res" = "acba" ] || err $LINENO

res=$($com <<< 'echo $( function hoge () { echo abc | rev ; } ; hoge )') 
[ "$res" = "cba" ] || err $LINENO

res=$($com <<< 'echo $( function hoge () { echo abc | rev ; } ; ( hoge ; hoge ) )') 
[ "$res" = "cba cba" ] || err $LINENO


cat << 'EOF' > $tmp 
f () {
	echo $1 $2 $3
	hoge=x
	echo $#
}

f a b c
echo $#
echo $hoge
EOF

res=$(cat $tmp  | $com x y z 1 2 3)
[ "$res" = "a b c
3
6
x" ] || err $LINENO


cat << 'EOF' > $tmp 
f () (
	echo $1 $2 $3
	hoge=x
	echo $#
)

hoge=y
f a b c
echo $#
echo $hoge
EOF

res=$(cat $tmp  | $com x y z 1 2 3)
[ "$res" = "a b c
3
6
y" ] || err $LINENO

cat << 'EOF' > $tmp 
f () {
	echo $1 $2 $3
	hoge=x
}

hoge=y
# Make f work in another process
f a b c | rev
echo $hoge
EOF

res=$(cat $tmp  | $com x y z)
[ "$res" = "c b a
y" ] || err $LINENO

res=$($com << 'EOF'
somefunc () {
	cat
	exit 1
}

echo a | somefunc 
echo $?
EOF
)
[ "$res" = "a
1" ] || err $LINENO

res=$($com <<< 'a(){ echo x; return ; echo b ; } ; a')
[ "$res" = "x" ] || err $LINENO

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

### GLOB FOR CASE ###

res=$($com <<< 'glob_test "a*" abcde')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "a*" z')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[abc]" a')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[!abc]" a')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[^abc]" a' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[abc][bcd][xy]" adx')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[abc][bcd][!xy]" adx' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[!abc!]" "!"' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'glob_test "[a-z]" "b"')
[ "$?" = "0" ] || err $LINENO

res=$($com <<< 'glob_test "[!a-c]" "b"')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'echo a || echo b || echo c')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< 'echo a && echo b || echo c')
[ "$res" = "a
b" ] || err $LINENO

res=$($com <<< 'echo a || echo b && echo c')
[ "$res" = "a
c" ] || err $LINENO

### WHILE ###

res=$($com <<< 'seq 3 | while read x ; do echo $x🎂 ; done')
[ "$res" = "1🎂
2🎂
3🎂" ] || err $LINENO

### CASE ###

res=$($com <<< 'case $- in *x*) echo x ;; *) echo no ;; esac')
[ "$res" = "no" ] || err $LINENO

res=$($com <<< 'case $- in *x*) ;; *) echo no ;; esac')
[ "$res" = "no" ] || err $LINENO

res=$($com -x <<< 'case $- in *x*) echo x ;; *) echo no ;; esac')
[ "$res" = "x" ] || err $LINENO

res=$($com <<< 'A=hoge ; case $A in *x*|*h*) echo aaa ;; *) echo no ;; esac')
[ "$res" = "aaa" ] || err $LINENO

res=$($com << 'EOF'
case xterm-color in
    xterm-color|*-256color) color_prompt=yes;;
esac
echo $color_prompt
EOF
)
[ "$res" = "yes" ] || err $LINENO

res=$($com << 'EOF'
case $- in 
	*x*) echo x ;;
	*) echo no ;;
esac
EOF
)
[ "$res" = "no" ] || err $LINENO

cat << EOF > $tmp 
echo hoge
EOF

res=$($com $tmp )
[ "$res" = "hoge" ] || err $LINENO

cat << EOF > $tmp 
#!$PWD/$com
echo hoge
EOF

chmod +x $tmp 
res=$($tmp )
[ "$res" = "hoge" ] || err $LINENO

echo OK $0
