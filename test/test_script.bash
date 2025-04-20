#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

tmp=/tmp/$$

cd $(dirname $0)
com=../target/release/sush

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

LANG=C ./lineno.bash 2> $tmp-bash
LANG=C ./lineno.sush 2> $tmp-sush

sed 's/sush/bash/g' $tmp-sush |
sed 's;../target/release;/bin;' |
diff $tmp-bash - || err $LINENO

cat << EOF > $tmp-script
#!../target/release/sush -xv

(
	echo a
	eeeee
)
EOF

chmod +x $tmp-script
$tmp-script |& grep 5:
[ $? -eq 0 ] || err $LINENO

cat << EOF > $tmp-script
#!../target/release/sush -xv

(

)
EOF

chmod +x $tmp-script
$tmp-script |& grep 5:
[ $? -eq 0 ] || err $LINENO

cat << 'EOF' > $tmp-script
#!../target/release/sush -xv

echo \
$LINENO \
$LINENO
EOF

chmod +x $tmp-script
res=$($tmp-script)
[ "$res" = "4 4" ] || err $LINENO

echo $0 >> ./ok
