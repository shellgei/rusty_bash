#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	rm -f $tmp-*
	exit 1
}

cd $(dirname $0)
com=../target/release/sush
tmp=/tmp/$$

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

res=$($com <<< 'declare -i n; n="1+1" ; echo $n')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( n ))')
[ "$res" = "0" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( (n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'echo $(( c=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( c=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'declare -i n; echo $(( c+=(n+1) ))')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'printf "%03x" 123')
[ "$res" = "07b" ] || err $LINENO

res=$($com <<< 'printf "%03X" 123')
[ "$res" = "07B" ] || err $LINENO

res=$($com <<< 'printf "%3X" 123')
[ "$res" = " 7B" ] || err $LINENO

res=$($com <<< 'printf "%-3X" 123')
[ "$res" = "7B " ] || err $LINENO

res=$($com <<< 'printf "%10s" 123')
[ "$res" = "       123" ] || err $LINENO

res=$($com <<< 'printf "%010s" 123')
[ "$res" = "       123" ] || err $LINENO

res=$($com <<< 'printf "%-10s" 123')
[ "$res" = "123       " ] || err $LINENO

res=$($com <<< 'printf "%010d" -123')
[ "$res" = "-000000123" ] || err $LINENO

res=$($com <<< 'printf "%f" -.3')
[ "$res" = "-0.300000" ] || err $LINENO

res=$($com <<< 'printf "%b" "aaa\nbbb"')
[ "$res" = "aaa
bbb" ] || err $LINENO

res=$($com <<< 'let a=1; echo $a')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'let a=1 b=0; echo $a $b $?')
[ "$res" = "1 0 1" ] || err $LINENO

res=$($com <<< 'let a== b=0; echo $a $b $?')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'let "c=$((1+1))"; echo $c $?')
[ "$res" = "2 0" ] || err $LINENO

res=$($com <<< 'let a=1; echo $a')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< '  function a { echo b ; } ; a')
[ "$res" = "b" ] || err $LINENO

res=$($com <<< 's="[0]" ; g="[0]" ;case $g in "$s") echo ok ;; esac')
[ "$res" = "ok" ] || err $LINENO

res=$($com <<< 'a=" a b c "; set 1${a}2 ; echo $#')
[ "$res" = "5" ] || err $LINENO

res=$($com <<< 'IFS=": " ; a=" a b c:"; set 1${a}2 ; echo $#')
[ "$res" = "5" ] || err $LINENO

res=$($com <<< 'IFS=":" ; a=" a b c:"; set 1${a}2 ; echo $#')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'IFS=":" ; a=" a b c:"; set "${a}" ; echo $#')
[ "$res" = "1" ] || err $LINENO

res=$($com <<< 'IFS=": "; x=" :"; set x $x; shift; echo "[$#]($1)"')
[ "$res" = "[1]()" ] || err $LINENO

res=$($com <<< 'IFS=": "; x=" a :  : b : "; set x $x; shift; echo "[$#]($1)($2)($3)"')
[ "$res" = "[3](a)()(b)" ] || err $LINENO

res=$($com <<< 'IFS=": "; x=" a : b :  : "; set x $x; shift; echo "[$#]($1)($2)($3)"')
[ "$res" = "[3](a)(b)()" ] || err $LINENO

res=$($com << 'AAA'
cat << "EOF"
abc
EOF
AAA
)
[ "$res" = 'abc' ] || err $LINENO

res=$($com -c 'echo ${10}' {0..10})
[ "$res" = '10' ] || err $LINENO

res=$($com <<< 'a="*"; echo "@($a)"')
[ "$res" = '@(*)' ] || err $LINENO

res=$($com <<< 'a=aaa; echo ${a[@]}')
[ "$res" = 'aaa' ] || err $LINENO

res=$($com <<< 'printf %q "()\""')
[ "$res" = '\(\)\"' ] || err $LINENO

res=$($com <<< "printf %q '@(|!(!(|)))'")
[ "$res" = '@\(\|\!\(\!\(\|\)\)\)' ] || err $LINENO

res=$($com <<< 'compgen -f -X "*test*" | grep test')
[ "$?" = "1" ] || err $LINENO

res=$($com <<< 'echo "ab `echo a`"')
[ "$res" = "ab a" ] || err $LINENO

res=$($com <<< 'printf -v __git_printf_supports_v %s yes; echo $__git_printf_supports_v' )
[ "$res" = "yes" ] || err $LINENO

res=$($com <<< 'printf -v __git_printf_supports_v -- %s yes; echo $__git_printf_supports_v' )
[ "$res" = "yes" ] || err $LINENO

res=$($com <<< 'printf "== <%s %s> ==\n" a b c' )
[ "$res" = "== <a b> ==
== <c > ==" ] || err $LINENO

res=$($com << 'EOF'
mkdir -p /tmp/$$
cd /tmp/$$
mkdir a b
touch a/{aa,ab}
touch b/{bb,bc}
ln -s a c
shopt -s globstar
echo 1: **
echo 2: **/
echo 3: **/*
echo 4: **/**/*
echo 5: a/**
echo 6: a/**/**
rm a/*
rm b/*
rm c
rmdir a b
rmdir /tmp/$$
EOF
)
[ "$res" = "1: a a/aa a/ab b b/bb b/bc c
2: a/ b/ c/
3: a a/aa a/ab b b/bb b/bc c
4: a a/aa a/ab b b/bb b/bc c
5: a/ a/aa a/ab
6: a a/aa a/ab" ] || err $LINENO

rm -f $tmp-*
echo $0 >> ./ok
exit

### issue 130 ###
### input-line.sh test of Bash ###

# It works.
cat << 'EOF' > $tmp-script
read a
echo @$a
EOF
chmod +x $tmp-script
res=$(bash << EOF
$com $tmp-script
OH
EOF
)
[ "$res" = "@OH" ] || err $LINENO

# It doesn't work.
# Maybe the exec-on-close is applied to
# the file discriptor of $com << EOF. 

chmod +x $tmp-script
res=$($com << EOF
$com $tmp-script
OH
EOF
)
[ "$res" = "@OH" ] || err $LINENO

### WHY ???????????? ###

#ueda@x1gen13:~/GIT/bash_for_sush_test/sush_test$ echo "a:b:" | ( IFS=" :" read x y; echo "($x)($y)" )
#(a)(b)
#ueda@x1gen13:~/GIT/bash_for_sush_test/sush_test$ echo "a:b::" | ( IFS=" :" read x y; echo "($x)($y)" )
#(a)(b::)

