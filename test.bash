#!/bin/bash -exv

trap "echo TEST NG; exit 1" EXIT

cargo build --release


com=$(dirname $0)/target/release/bash_r

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ]

### ARG TEST ###
res=$($com <<- EOF
echo 'a'
EOF
)
[ "$res" = "a" ]

trap "" EXIT
echo TEST OK
