#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	rm -f $tmp-*
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO


cd $(dirname $0)
com=../target/release/sush
tmp=/tmp/$$

res=$($com <<< 'echo "aaa\bb" | ( read -r a ; echo $a )' )
[ "$res" = "aaa\bb" ] || err $LINENO

res=$($com <<< 'echo "aaa\bb" | ( read a ; echo $a )' )
[ "$res" = "aaabb" ] || err $LINENO

res=$($com << 'EOF'
echo 'aaa\
bb' | ( read a ; echo $a )
EOF
)
[ "$res" = "aaabb" ] || err $LINENO

res=$($com << 'EOF'
echo 'aaa\
bb' | ( read -r a ; echo $a )
EOF
)
[ "$res" = 'aaa\' ] || err $LINENO

res=$($com <<< 'echo {2147483650..2147483655}')
[ "$res" = "2147483650 2147483651 2147483652 2147483653 2147483654 2147483655" ] || err $LINENO

res=$($com << 'AAA'
while read a b ; do echo $a _ $b ; done << EOF
A B
A ()
t fofo                *(f*(o))
EOF
AAA
)
[ "$res" = "A _ B
A _ ()
t _ fofo *(f*(o))" ] || err $LINENO

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
