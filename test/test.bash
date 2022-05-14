#!/bin/bash -exv

trap "echo TEST NG; exit 1" EXIT

cargo build --release

cd $(dirname $0)

com=../target/release/bash_r

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ]

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ]

#### ARG TEST ###

res=$($com <<< 'echo aaa"bbb"')
[ "$res" = "aaabbb" ]

res=$($com << 'EOF'
echo 'a' "b  b" cc  c
EOF
)
[ "$res" = "a b  b cc c" ]

res=$($com << 'EOF'
echo "\"" "\\" a\ \ bc
EOF
)
[ "$res" = '" \ a  bc' ]

res=$($com <<< 'echo hoge"hoge";')
[ "$res" = "hogehoge" ]

# brace expansion

res=$($com << 'EOF'
echo {a}
EOF
)
[ "$res" = '{a}' ]

res=$($com << 'EOF'
echo {a,b}{cc,dd}
EOF
)
[ "$res" = 'acc add bcc bdd' ]

res=$($com << 'EOF'
echo "{a,b}{cc,dd}"
EOF
)
[ "$res" = '{a,b}{cc,dd}' ]

res=$($com << 'EOF'
echo あ{cc,いうえお}
EOF
)
[ "$res" = 'あcc あいうえお' ]

res=$($com << 'EOF'
echo {a,b}{c,d}へ{e,f}
EOF
)
[ "$res" = 'acへe acへf adへe adへf bcへe bcへf bdへe bdへf' ]

# glob test

res=$($com << 'EOF'
ls *.bash
EOF
)
[ "$res" = "test.bash" ]

trap "" EXIT
echo TEST OK
