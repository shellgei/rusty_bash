#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

tmp=/tmp/$$

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

LANG=C ./lineno.bash 2> $tmp-bash
LANG=C ./lineno.sush 2> $tmp-sush

sed 's/sush/bash/g' $tmp-sush |
diff $tmp-bash - || err $LINENO


echo $0 >> ./ok
