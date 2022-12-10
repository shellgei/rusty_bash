#!/bin/bash -e
# SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $1
	exit 1
}

cargo build --release || err $LINENO

cd $(dirname $0)

com=../target/release/rusty_bash

./test_simple_command.bash || err test_simple_command.bash
./test_others.bash         || err test_others.bash
./test_jobs.bash           || err test_jobs.bash

echo TEST OK
