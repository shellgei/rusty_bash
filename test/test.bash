#!/bin/bash -exv
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

trap "echo TEST NG; exit 1" EXIT

cargo build --release

cd $(dirname $0)

com=../target/release/rusty_bash

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ]

### POSITIONAL PARAMETERS ###

cat << 'EOF' > /tmp/.rusty_bash
echo $1 $2 $3
EOF

res=$(cat /tmp/.rusty_bash | $com a b c)
[ "$res" = "a b c" ]

#### ARG TEST ###

res=$($com <<< 'echo aaa"bbb"')
[ "$res" = "aaabbb" ]

res=$($com << 'EOF'
echo 'a' "b  b" cc  c
EOF
)
[ "$res" = "a b  b cc c" ]

res=$($com << 'EOF'
echo "\"" "\\" a\ \ bc
EOF
)
[ "$res" = '" \ a  bc' ]

res=$($com <<< 'echo "\a\n\$\`\{\}"')
[ "$res" = '\a\n$`{}' ]

res=$($com << 'EOF'
echo "a'b'c"
EOF
)
[ "$res" = "a'b'c" ]

res=$($com <<< 'echo hoge"hoge";')
[ "$res" = "hogehoge" ]

res=$($com <<< 'echo )' || true)
[ "$res" = "" ]

# brace expansion

res=$($com << 'EOF'
echo {a}
EOF
)
[ "$res" = '{a}' ]

res=$($com << 'EOF'
echo {a,b}{cc,dd}
EOF
)
[ "$res" = 'acc add bcc bdd' ]

res=$($com << 'EOF'
echo "{a,b}{cc,dd}"
EOF
)
[ "$res" = '{a,b}{cc,dd}' ]

res=$($com << 'EOF'
echo あ{cc,いうえお}
EOF
)
[ "$res" = 'あcc あいうえお' ]

res=$($com << 'EOF'
echo {a,b}{c,d}へ{e,f}
EOF
)
[ "$res" = 'acへe acへf adへe adへf bcへe bcへf bdへe bdへf' ]

res=$($com << 'EOF'
echo {,b,c}{a,b}
EOF
)
[ "$res" = 'a b ba bb ca cb' ]

res=$($com << 'EOF'
echo {a,"b,c",'d,e',f}
EOF
)
[ "$res" = 'a b,c d,e f' ]

res=$($com <<< 'echo {a,b{c,d},e}')
[ "$res" = "a bc bd e" ]

res=$($com <<< 'echo {a,*}zzzzz')
[ "$res" = "azzzzz *zzzzz" ]

res=$($com <<< 'echo {')
[ "$res" = '{' ]

res=$($com <<< 'echo a{b{c')
[ "$res" = 'a{b{c' ]

res=$($com <<< 'echo a{b{c}')
[ "$res" = 'a{b{c}' ]

res=$($com <<< 'echo a{b{}')
[ "$res" = 'a{b{}' ]

res=$($com <<< 'echo }')
[ "$res" = '}' ]

# glob test

res=$($com << 'EOF'
ls *.bash
EOF
)
[ "$res" = "test.bash" ]

res=$($com <<< 'echo "*"')
[ "$res" = "*" ]

res=$($com <<< 'echo /')
[ "$res" = "/" ]

#The following checks trivial difference between bash and this.
#$com <<< 'echo //*' | grep -F '//' 
#$com <<< 'echo /*////' | grep -Fv '//'

# command substitution

res=$($com <<< 'echo $(echo hoge)hoge')
[ "$res" = "hogehoge" ]

res=$($com <<< 'echo hoge$(echo hoge)')
[ "$res" = "hogehoge" ]

res=$($com <<< 'echo "hoge$(echo hoge)"')
[ "$res" = "hogehoge" ]

res=$($com <<< 'echo "hoge$(echo hoge; echo hoge)"')
[ "$res" = "hogehoge
hoge" ]

res=$($com <<< 'echo "$(seq 2)"')
[ "$res" = "1
2" ]

res=$($com <<< 'echo a"$(seq 4)"b')
[ "$res" = "a1
2
3
4b" ]

res=$($com <<< 'echo "$(seq 3)"{a,b}')
[ "$res" = "1
2
3a 1
2
3b" ]

res=$($com <<< 'echo {a,"$(seq 3)"}b')
[ "$res" = "ab 1
2
3b" ]

res=$($com <<< 'echo {a,$(seq 2)"$(seq 2)"{$(seq 2),"$(seq 2)"}}')
[ "$res" = "a 1 21
21 2 1 21
21
2" ]

res=$($com <<< 'echo $(seq 3){a,b}')
[ "$res" = "1 2 3a 1 2 3b" ]

res=$($com <<< 'cd /;echo "$(pwd)x"') #internal command
[ "$res" = "/x" ]

$com <<< 'ls $(echo / /)'
[ $? -eq 0 ]

res=$($com <<< 'echo {$(seq 5)}')
[ "$res" = "{1 2 3 4 5}" ]

res=$($com <<< 'echo {$(seq 3),$(seq 3)')
[ "$res" = "{1 2 3,1 2 3" ]


res=$($com <<< 'echo $( echo abc | (rev) )') 
[ "$res" = "cba" ]

res=$($com <<< 'echo $( echo a ;  ( echo b ; echo c ) )')
[ "$res" = "a b c" ]

# expansion of tilde

res=$($com <<< 'echo ~')
[ "$res" = "$HOME" ]

res=$($com <<< 'echo "~"')
[ "$res" = "~" ]

res=$($com <<< 'echo ~/')
[ "$res" = "$HOME/" ]

res=$($com <<< 'echo ~a')
[ "$res" = "~a" ]

res=$($com <<< 'echo ~*')
[ "$res" = "~*" ]

user=$(tail -n 1 /etc/passwd | awk -F: '{print $1}')
home=$(tail -n 1 /etc/passwd | awk -F: '{print $(NF-1)}')

res=$($com <<< "echo ~$user")
[ "$res" = "$home" ]

res=$($com <<< "echo {~$user,a}")
[ "$res" = "$home a" ]

### DIRECTORY TEST ###

res=$($com << 'EOF'
cd /
pwd
EOF
)
[ "$res" = "/" ]

### COMMAND BOUNDARY TEST ###

res=$($com <<< 'echo hoge;echo hoge')
[ "$res" = "hoge
hoge" ]

### COMMENT TEST ###

res=$($com << 'EOF'
echo hello #HEHEHEHEHEあいうえお
echo world
EOF
)
[ "$res" = 'hello
world' ]

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
aあいうえお'b'c" ]

res=$($com <<< 'a={a,b}{c,d};echo $a')
[ "$res" = "{a,b}{c,d}" ]

res=$($com << 'EOF'
abc=あいうえお
def=${abc}かきくけこ
echo $def
EOF
)
[ "$res" = "あいうえおかきくけこ" ]

res=$($com <<< 'LANG=C TZ= date -d 2000-01-01')
[ "$res" = "Sat Jan  1 00:00:00 UTC 2000" ]

res=$($com << 'EOF'
LANG=C
TZ= date -d 2000-01-01
EOF
)
[ "$res" = "Sat Jan  1 00:00:00 UTC 2000" ]

# special variable

res=$($com <<< 'ls aaaaaaa; echo $?')
[ "$res" = "2" ]

### REDIRECTION ###

res=$($com << 'EOF'
echo text > /tmp/.rusty_bash
wc < /tmp/.rusty_bash
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "1 1 5" ]

res=$($com << 'EOF'
echo text 1> /tmp/.rusty_bash
wc 0< /tmp/.rusty_bash
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "1 1 5" ]

$com << 'EOF' | grep 'aaaa'
ls aaaaaaaaaaaaaaaaaaaaaa 2> /tmp/.rusty_bash
cat /tmp/.rusty_bash 
rm /tmp/.rusty_bash
EOF

res=$($com << 'EOF' 
ls -d / aaaaaaaaaaaaaaaaaaaa &> /tmp/.rusty_bash
wc -l < /tmp/.rusty_bash
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "2" ]

res=$($com << 'EOF' 
ls -d /hogehgoe 2>&1
EOF
) || true
[ "$(echo $res | wc -l)" = "1" ]

res=$($com << 'EOF' 
echo $(ls /aaaa 2>&1)x
EOF
)
echo "$res" | grep x | grep ls

res=$($com << 'EOF'
ls aaaaaaaaaaaaaaaaaaa > /tmp/.rusty_bash 2>&1
wc -l < /tmp/.rusty_bash
rm /tmp/.rusty_bash
EOF
)

[ "$res" = "1" ]

res=$($com << 'EOF'
2>/tmp/.rusty_bash echo hoge
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "hoge" ]

res=$($com << 'EOF'
echo 2>/tmp/.rusty_bash hoge
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "hoge" ]

res=$($com << 'EOF'
A=B >/tmp/.rusty_bash C=D echo hoge
cat /tmp/.rusty_bash
rm /tmp/.rusty_bash
EOF
)
[ "$res" = "hoge" ]

### PIPELINE ###

res=$($com <<< 'echo abc | rev')
[ "$res" = "cba" ]

res=$($com <<< 'echo abc | rev | tr abc def')
[ "$res" = "fed" ]

### COMPOUND COMMAND ###

res=$($com <<< '(echo hoge)')
[ "$res" = "hoge" ]

res=$($com <<< '{echo hoge; }')
[ "$res" = "hoge" ]

res=$($com <<< '(echo hoge;echo hoge)')
[ "$res" = "hoge
hoge" ]

res=$($com <<< '{echo hoge;echo hoge ; }')
[ "$res" = "hoge
hoge" ]

res=$($com <<< '(echo hoge | rev;echo hoge)')
[ "$res" = "egoh
hoge" ]

res=$($com <<< 'echo abc | ( echo a ; rev ) | tr -d \\n')
[ "$res" = "acba" ]

res=$($com <<< '{echo hoge | rev;echo hoge ; }')
[ "$res" = "egoh
hoge" ]

res=$($com <<< 'echo abc | { echo a ; rev ; } | tr -d \\n')
[ "$res" = "acba" ]

res=$($com <<< '(A=B);echo $A')
[ "$res" = "" ]

res=$($com <<< '{A=B ; };echo $A')
[ "$res" = "B" ]

res=$($com <<< 'echo abc | (rev)')
[ "$res" = "cba" ]

res=$($com <<< '(echo abc) | rev')
[ "$res" = "cba" ]

res=$($com << 'EOF'
(ls aaaaa) 2> /tmp/.rusty_bash
cat /tmp/.rusty_bash | wc -l
rm /tmp/.rusty_bash
EOF
)
[ "$res" -ge 1 ]

res=$($com <<< '{ echo } ; }')
[ "$res" = "}" ]

### MULTILINE INPUT ###

res=$($com << 'EOF'
echo a \
b \
c
EOF
)
[ "$res" = "a b c" ]

res=$($com << 'EOF'
ec\
ho a\
b\
c
EOF
)
[ "$res" = "abc" ]

res=$($com << 'EOF'
(
echo a
echo b
)
EOF
)
[ "$res" = "a
b" ]

res=$($com << 'EOF'
{
echo a
echo b
}
EOF
)
[ "$res" = "a
b" ]

res=$($com << 'EOF'
echo abc |
rev
EOF
)
[ "$res" = "cba" ]

### FUNCTION ###

res=$($com << 'EOF'
somefunc () {
	echo a
}

somefunc
EOF
)
[ "$res" = "a" ]

res=$($com << 'EOF'
function somefunc () {
	echo a
}

somefunc
somefunc
EOF
)
[ "$res" = "a
a" ]

res=$($com << 'EOF'
somefunc (    ) {
	echo abc
}

somefunc | rev
EOF
)
[ "$res" = "cba" ]

res=$($com << 'EOF'
somefunc () {
	echo a
	rev
}

echo abc | somefunc | tr -d '\n'
EOF
)
[ "$res" = "acba" ]

res=$($com <<< 'echo $( function hoge () { echo abc | rev ; } ; hoge )') 
[ "$res" = "cba" ]

res=$($com <<< 'echo $( function hoge () { echo abc | rev ; } ; ( hoge ; hoge ) )') 
[ "$res" = "cba cba" ]


cat << 'EOF' > /tmp/.rusty_bash
f () {
	echo $1 $2 $3
	hoge=x
}

f a b c
echo $hoge
EOF

res=$(cat /tmp/.rusty_bash | $com x y z)
[ "$res" = "a b c
x" ]

cat << 'EOF' > /tmp/.rusty_bash
f () {
	echo $1 $2 $3
	hoge=x
}

hoge=y
# Make f work in another process
f a b c | rev
echo $hoge
EOF

res=$(cat /tmp/.rusty_bash | $com x y z)
[ "$res" = "c b a
y" ]

trap "" EXIT
echo TEST OK
