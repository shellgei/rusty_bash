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
