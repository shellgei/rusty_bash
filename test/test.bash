#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

export SUSH_COMPAT_TEST_MODE=0

cd $(dirname $0)
com=../target/release/sush

cargo build --release || err $LINENO
cargo --version

: > error
: > ok

./test_case.bash nobuild &
./test_others.bash nobuild &
./test_redirects.bash nobuild &
./test_calculation.bash nobuild &
./test_compound.bash nobuild &
./test_script.bash nobuild &
./test_job.bash nobuild &
./test_brace.bash nobuild &
./test_builtins.bash nobuild &
./test_options.bash nobuild &
./test_parameters.bash nobuild &
./test_glob.bash nobuild &
./test_ansi_c_quoting.bash nobuild &
./test_fixed.bash nobuild &
./test_param_substitutions.bash nobuild &

wait 

head ./ok ./error

[ $(cat ./error | wc -l) == "0" ]  || err $LINENO

echo OK $0
