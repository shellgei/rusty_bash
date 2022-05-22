#!/bin/bash -exv

trap "echo TEST NG; exit 1" EXIT

cargo build --release

cd $(dirname $0)

com=../target/release/rusty_bash

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

res=$($com << 'EOF'
echo "a'b'c"
EOF
)
[ "$res" = "a'b'c" ]

res=$($com <<< 'echo hoge"hoge";')
[ "$res" = "hogehoge" ]

# variable

res=$($com <<< 'echo ${SHELL}')
[ "$res" = "rustybash" ]

res=$($com <<< 'echo $SHELL')
[ "$res" = "rustybash" ]

res=$($com << 'EOF'
echo "a${SHELL}'b'c"
EOF
)
[ "$res" = "arustybash'b'c" ]

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

res=$($com << 'EOF'
echo {,b,c}{a,b}
EOF
)
[ "$res" = 'a b ba bb ca cb' ]

res=$($com << 'EOF'
echo {a,"b,c",'d,e',f}
EOF
)
[ "$res" = 'a b,c d,e f' ]

res=$($com <<< 'echo {a,b{c,d},e}')
[ "$res" = "a bc bd e" ]

res=$($com <<< 'echo {a,*}zzzzz')
[ "$res" = "azzzzz *zzzzz" ]

# glob test

res=$($com << 'EOF'
ls *.bash
EOF
)
[ "$res" = "test.bash" ]

res=$($com <<< 'echo "*"')
[ "$res" = "*" ]

### DIRECTORY TEST ###

res=$($com << 'EOF'
cd /
pwd
EOF
)
[ "$res" = "/" ]

### COMMAND BOUNDARY TEST ###

res=$($com <<< 'echo hoge;echo hoge')
[ "$res" = "hoge
hoge" ]

### COMMENT TEST ###

res=$($com << 'EOF'
echo hello #HEHEHEHEHEあいうえお
echo world
EOF
)
[ "$res" = 'hello
world' ]

trap "" EXIT
echo TEST OK
