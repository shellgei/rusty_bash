#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

cd $(dirname $0)
com=../target/release/sush

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

### JOB TEST ###

res=$($com <<< 'sleep 1 & sleep 2 & sleep 3 & jobs')
echo "$res" | grep -F '[1] ' || err $LINENO
echo "$res" | grep -F '[2]- ' || err $LINENO
echo "$res" | grep -F '[3]+ ' || err $LINENO

res=$($com <<< 'sleep 5 | rev | cat & sleep 1 ; killall -SIGSTOP cat ; jobs')
echo "$res" | grep Stopped || err $LINENO

echo $0 >> ./ok
