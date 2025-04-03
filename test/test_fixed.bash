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

