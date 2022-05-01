#!/bin/bash -exv

cargo build --release


com=$(dirname $0)/target/release/bash_r


res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ]
