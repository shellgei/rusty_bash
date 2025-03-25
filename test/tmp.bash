#!/bin/bash 
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

cd $(dirname $0)
com=../target/release/sush
tmp=/tmp/$$

cat << 'EOF' > $tmp-script
read a
echo @$a
EOF
res=$(bash << EOF
$com $tmp-script
OH
EOF
)
