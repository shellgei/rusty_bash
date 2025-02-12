#!/bin/bash -xv
# SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
# SPDX-License-Identifier: BSD-3-Clause

err () {
	echo $0 >> ./error
	echo "ERROR!" FILE: $0, LINENO: $1
	exit 1
}

[ "$1" == "nobuild" ] || cargo build --release || err $LINENO

cd $(dirname $0)
com=../target/release/sush

### SIMPLE COMMAND TEST ###

res=$($com <<< 'echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< ' echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '	echo hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge;')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< '! eeee' )
[ "$?" = "0" ] || err $LINENO

res=$($com <<< '! echo' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< '! cd' )
[ "$?" = "1" ] || err $LINENO

res=$($com <<< '!' )
[ "$?" = "1" ] || err $LINENO

### PARAMETER TEST ###
#
res=$($com <<< '_AAA=3 ; echo $_AAA' )
[ "$res" = "3" ] || err $LINENO

res=$($com <<< 'echo ${A:-abc}' )
[ "$res" = "abc" ] || err $LINENO

res=$($com <<< 'echo ${A:-abc}; echo $A' )
[ "$res" = "abc" ] || err $LINENO

res=$($com <<< 'echo ${A:=abc}; echo $A' )
[ "$res" = "abc
abc" ] || err $LINENO

res=$($com <<< 'echo ${A:="aaa
bbb"}
echo "$A"' )
[ "$res" = "aaa bbb
aaa
bbb" ] || err $LINENO

res=$($com <<< 'echo ${A:?error}' )
[ "$?" = "1" ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< '(echo ${A:?eRRor}) |& cat' )
echo "$res" | grep -q eRRor || err $LINENO

res=$($com <<< 'A=123; echo ${A:?eRRor}' )
[ "$res" = "123" ] || err $LINENO

res=$($com <<< 'A= ; echo ${A:+set}' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=aaa ; echo ${A:+set}' )
[ "$res" = "set" ] || err $LINENO

res=$($com <<< 'A=aaa ; echo ${A:+"set
ok"}' )
[ "$res" = "set
ok" ] || err $LINENO

res=$($com <<< 'A=aaa ; echo ${A- - - - -}' )
[ "$res" = "aaa" ] || err $LINENO

res=$($com <<< 'A=aaa ; echo ${A+- - - - bbb}' )
[ "$res" = "- - - - bbb" ] || err $LINENO

res=$($com <<< 'A= ; echo ${A+- - - - bbb}' )
[ "$res" = "- - - - bbb" ] || err $LINENO

res=$($com <<< 'echo ${A:-   abc}' )
[ "$res" = "abc" ] || err $LINENO

res=$($com <<< 'echo ${A:-abc def}' )
[ "$res" = "abc def" ] || err $LINENO

res=$($com <<< 'echo ${A:-abc   def}' )
[ "$res" = "abc def" ] || err $LINENO

res=$($com <<< 'B=あ ; echo ${A:-$B def}' )
[ "$res" = "あ def" ] || err $LINENO

res=$($com <<< 'B=あ ; echo ${A:-$B
def}' )
[ "$res" = "あ def" ] || err $LINENO

res=$($com <<< 'B=あ ; echo ${A:-"$B
def"}' )
[ "$res" = "あ
def" ] || err $LINENO

res=$($com <<< 'A=aaa; B= ; echo ${B+$A}' )
[ "$res" = "aaa" ] || err $LINENO

res=$($com <<< 'A=aaa; echo ${B+$A}' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=aaa; B=b ; echo ${B+$A}' )
[ "$res" = "aaa" ] || err $LINENO

# offset

res=$($com <<< 'A=abc; echo ${A:1}' )
[ "$res" = "bc" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:2}' )
[ "$res" = "うえお" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:1 + 1 }' )
[ "$res" = "うえお" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:1 + 1:1}' )
[ "$res" = "う" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:1 + 1:2}' )
[ "$res" = "うえ" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:1 + 1:9}' )
[ "$res" = "うえお" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:1 + 1:}' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:}' )
[ "$?" = 1 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=あ; echo ${A: }' )
[ "$res" = "あ" ] || err $LINENO

res=$($com <<< 'A=あいうえお; echo ${A:6}' )
[ "$res" = "" ] || err $LINENO

res=$($com <<< 'A=usr/local/bin/bash; echo ${A#*/}' )
[ "$res" = "local/bin/bash" ] || err $LINENO

res=$($com <<< 'A=usr/local/bin/bash; echo ${A##*/}' )
[ "$res" = "bash" ] || err $LINENO

res=$($com <<< 'A=usr/local/bin/bash; echo ${A%/*}' )
[ "$res" = "usr/local/bin" ] || err $LINENO

res=$($com <<< 'A=usr/local/bin/bash; echo ${A%%/*}' )
[ "$res" = "usr" ] || err $LINENO

res=$($com <<< 'A="あいう うえお"; echo ${A#*う う}' )
[ "$res" = "えお" ] || err $LINENO

res=$($com <<< 'A="[["; echo ${A%%[[(]}' )
[ "$res" = "[" ] || err $LINENO

# replace

res=$($com -c 'A="あいう うえお"; echo ${A/あ/}' )
[ "$res" = "いう うえお" ] || err $LINENO

res=$($com -c 'A="あいう うえお"; echo ${A/あ//}' )
[ "$res" = "/いう うえお" ] || err $LINENO

res=$($com -c 'A="あいう うえお"; echo ${A/い/え}' )
[ "$res" = "あえう うえお" ] || err $LINENO

res=$($com -c 'A="あいう うえお"; echo ${A/いう/えええeee}' )
[ "$res" = "あえええeee うえお" ] || err $LINENO

res=$($com -c 'A="あいう うえお"; echo ${A//う/えええeee}' )
[ "$res" = "あいえええeee えええeeeえお" ] || err $LINENO

res=$($com -c 'A="あいう いうえお"; echo ${A//いう/えええeee}' )
[ "$res" = "あえええeee えええeeeえお" ] || err $LINENO

res=$($com -c 'A="あいう いうえお"; echo ${A/#いう/えええeee}' )
[ "$res" = "あいう いうえお" ] || err $LINENO

res=$($com -c 'A="あいう いうえお"; echo ${A/#あいう/えええeee}' )
[ "$res" = "えええeee いうえお" ] || err $LINENO

res=$($com -c 'A="あいう いうえお"; echo ${A/%えお/えええeee}' )
[ "$res" = "あいう いうえええeee" ] || err $LINENO

res=$($com -c 'A="あいうえお いうえお"; echo ${A/%えお/えええeee}' )
[ "$res" = "あいうえお いうえええeee" ] || err $LINENO

res=$($com -c 'A="あいうえお"; echo ${A/%あ/えええeee}' )
[ "$res" = "あいうえお" ] || err $LINENO

res=$($com -c 'echo ${@[0]}' )
[ $? = 1 ] || err $LINENO
[ "$res" = "" ] || err $LINENO

### IRREGULAR INPUT TEST ###

res=$($com <<< 'eeeeeecho hoge')
[ "$?" = 127 ] || err $LINENO

res=$($com <<< ';')
[ "$?" = 2 ] || err $LINENO

res=$($com <<< ';a')
[ "$?" = 2 ] || err $LINENO

### PIPELINE ###

res=$($com <<< 'seq 10 | rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |
	rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |    

	  rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |  #コメントだよ

#コメントだよ
    #こめんとだよ

	  rev | tail -n 1')
[ "$res" = "01" ] || err $LINENO

res=$($com <<< 'seq 10 |   | head -n 1')
[ "$?" = "2" ] || err $LINENO

### COMMENT ###

res=$($com <<< 'echo a #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '
#comment comment
   #comment comment
echo a #aaaaa
#comment comment
')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a) #aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '(echo a)#aaaaa')
[ "$res" = "a" ] || err $LINENO

res=$($com <<< '{ echo a; }#aaaaa')
[ "$res" != "a" ] || err $LINENO

res=$($com <<< '{ echo a; } #aaaaa')
[ "$res" = "a" ] || err $LINENO

### NEW LINE ###

res=$($com <<< 'e\
c\
ho hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'e\
c\
ho \
hoge')
[ "$res" = "hoge" ] || err $LINENO

res=$($com <<< 'echo hoge |\
rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< 'echo hoge |\
& rev')
[ "$res" = "egoh" ] || err $LINENO

res=$($com <<< ' (seq 3; seq 3) | grep 3 | wc -l | tr -dc 0-9')
[ "$res" = "2" ] || err $LINENO

res=$($com <<< 'ls |  | rev')
[ "$?" == "2" ] || err $LINENO

### JOB PARSE TEST ###

res=$($com <<< '&& echo a')
[ "$?" == "2" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo a
&& echo b')
[ "$?" == "2" ] || err $LINENO

res=$($com <<< 'echo a &\
& echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&\
echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&
echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a &&



echo b')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo a ||
echo b')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'echo a \
&& echo b')
[ "$res" == "a
b" ] || err $LINENO

# double quotation

res=$($com <<< 'echo "*"')
[ "$res" == "*" ] || err $LINENO

res=$($com <<< 'echo "{a,{b},c}"')
[ "$res" == "{a,{b},c}" ] || err $LINENO

export RUSTY_BASH_A='a
b'
res=$($com <<< 'echo "$RUSTY_BASH_A"')
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<< 'echo "$BASH{PID,_SUBSHELL}"')
[ "$res" == "{PID,_SUBSHELL}" ] || err $LINENO

res=$($com <<< 'echo "\$HOME"')
[ "$res" == '$HOME' ] || err $LINENO

res=$($com <<< 'echo "\a"')
[ "$res" == '\a' ] || err $LINENO

res=$($com <<< 'echo "\\"')
[ "$res" == '\' ] || err $LINENO

res=$($com <<< 'echo "a   b"')
[ "$res" == 'a   b' ] || err $LINENO

res=$($com <<< 'echo "a
b
c"')
[ "$res" == 'a
b
c' ] || err $LINENO

res=$($com <<< 'echo "')
[ "$?" == 2 ] || err $LINENO

res=$($com <<< 'echo "" a')
[ "$res" == " a" ] || err $LINENO

res=$($com <<< 'set a b c; echo a"$@"c')
[ "$res" == "aa b cc" ] || err $LINENO

res=$($com <<< 'set a b c; echo $#')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'set a b c; A=( A"$@"C ); echo ${A[0]}')
[ "$res" == "Aa" ] || err $LINENO

res=$($com <<< 'set a b c; A=( A"$@"C ); echo ${A[2]}')
[ "$res" == "cC" ] || err $LINENO

res=$($com <<< 'set a b c; A=( A"$*"C ); echo ${A[0]}')
[ "$res" == "Aa b cC" ] || err $LINENO

res=$($com <<< 'set a b c; A=( A$*C ); echo ${A[1]}')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'set a; A=( A"$@"C ); echo ${A[0]}')
[ "$res" == "AaC" ] || err $LINENO

res=$($com <<< 'A=( A"$@"C ); echo ${A[0]}')
[ "$res" == "AC" ] || err $LINENO

res=$($com <<< 'set あ; echo a"$@"c')
[ "$res" == "aあc" ] || err $LINENO

res=$($com <<< 'set あ い; echo a"$@"c')
[ "$res" == "aあ いc" ] || err $LINENO

res=$($com <<< 'echo a"$@"c')
[ "$res" == "ac" ] || err $LINENO

# single quoted

res=$($com <<< "echo '' a")
[ "$res" == " a" ] || err $LINENO

### WHILE TEST ###

res=$($com <<< 'touch /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "wait" ] || err $LINENO

res=$($com <<< 'rm -f /tmp/rusty_bash ; while [ -f /tmp/rusty_bash ] ; do echo wait ; rm /tmp/rusty_bash ; done')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'while false ; do echo do not come here ; done')
[ "$?" == 0 ] || err $LINENO
[ "$res" == "" ] || err $LINENO

### ARG TEST ###

# substitution

res=$($com <<< 'A=BBB; echo $A')
[ "$res" == "BBB" ] || err $LINENO

res=$($com <<< 'A=BBB echo ok')
[ "$res" == "ok" ] || err $LINENO

res=$($com <<< 'A=BBB B= echo ok')
[ "$res" == "ok" ] || err $LINENO

res=$($com <<< 'A=BBB $(); echo $A')
[ "$res" == "BBB" ] || err $LINENO

res=$($com <<< 'A=BBB $(echo); echo $A')
[ "$res" == "BBB" ] || err $LINENO

res=$($com <<< 'A=BBB bash -c "echo \$A"')
[ "$res" == "BBB" ] || err $LINENO

res=$($com <<< 'A=BBB B=CCC bash -c "echo \$A \$B"')
[ "$res" == "BBB CCC" ] || err $LINENO

res=$($com <<< 'A=A$(echo BBB)C; echo $A')
[ "$res" == "ABBBC" ] || err $LINENO

res=$($com <<< 'A={a,b}; echo $A')
[ "$res" == "{a,b}" ] || err $LINENO

res=$($com <<< 'A=/*; echo $A | grep -q "*"')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=/*; echo $A | grep -q "etc"')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'A=${ }; echo NG')
[ "$ref" != "NG" ] || err $LINENO

res=$($com <<< 'A=${ }')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'A=B cd ; echo $A')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=(a b) cd ; echo ${A[0]}')
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=aaa ; A+=bbb ; echo $A')
[ "$res" == "aaabbb" ] || err $LINENO

res=$($com <<< 'A=(aaa bbb) ; A+=(ccc ddd) ; echo ${A[@]}')
[ "$res" == "aaa bbb ccc ddd" ] || err $LINENO

# escaping

res=$($com <<< "echo a\ \ \ a")
[ "$res" == "a   a" ] || err $LINENO

res=$($com <<< 'echo \(')
[ "$res" == "(" ] || err $LINENO

# quotation

res=$($com <<< "echo 'abc'")
[ "$res" == "abc" ] || err $LINENO

res=$($com <<< "echo 'abあいうc'")
[ "$res" == "abあいうc" ] || err $LINENO

res=$($com <<< "echo 123'abc'")
[ "$res" == "123abc" ] || err $LINENO

res=$($com <<< "echo 123'abc'def")
[ "$res" == "123abcdef" ] || err $LINENO

# parameter expansion

res=$($com <<< 'echo $')
[ "$res" == "$" ] || err $LINENO

res=$($com <<< 'echo $?')
[ "$res" == "0" ] || err $LINENO

res=$($com <<< 'echo ${?}')
[ "$res" == "0" ] || err $LINENO

res=$($com <<< 'ls aaaaaaaa ; echo $?')
[ "$res" != "0" ] || err $LINENO

res=$($com <<< 'echo $BASH{PID,_SUBSHELL} | sed -E "s@[0-9]+@num@"')
[ "$res" == "num 0" ] || err $LINENO

res=$($com <<< 'echo ${BASHPID} ${BASH_SUBSHELL} | sed -E "s@[0-9]+@num@"')
[ "$res" == "num 0" ] || err $LINENO

res=$($com <<< 'echo ${ ')
[ "$?" == "2" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo ${ A}')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo ${A }')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo ${_A32523j2}')
[ "$?" == "0" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo ${_A32*523j2}')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo ${$,$} | grep "[^0-9]"')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'set a b c;echo $@')
[ "$res" == "a b c" ] || err $LINENO

res=$($com <<< 'A=あいうえおX; echo ${#A}')
[ "$res" == "6" ] || err $LINENO

res=$($com <<< 'A=(aaa bbbb); echo ${#A}; echo ${#A[1]}; echo ${#A[@]}')
[ "$res" == "3
4
2" ] || err $LINENO

# tilde

res=$($com <<< 'echo ~ | grep -q /')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo ~root')
[ "$res" == "/root" -o "$res" == "/var/root" ] || err $LINENO

res=$($com <<< 'cd /; cd /etc; echo ~+; echo ~-')
[ "$res" == "/etc
/" ] || err $LINENO


# split

export RUSTY_BASH_A='a
b'
res=$($com <<< 'echo $RUSTY_BASH_A')
[ "$res" == "a b" ] || err $LINENO

export RUSTY_BASH_A='a
b'
res=$($com <<< 'echo $RUSTY_BASH_A$RUSTY_BASH_A')
[ "$res" == "a ba b" ] || err $LINENO

export RUSTY_BASH_A='a
b'
res=$($com <<< 'echo ${RUSTY_BASH_A}c')
[ "$res" == "a bc" ] || err $LINENO

export RUSTY_BASH_A='a
b
'
res=$($com <<< 'echo ${RUSTY_BASH_A}c')
[ "$res" == "a b c" ] || err $LINENO

res=$($com <<< 'mkdir -p tmp; cd tmp; echo .* | grep -F ". .."; cd ..; rmdir tmp')
[ "$res" == '' ] || err $LINENO

res=$($com <<< 'mkdir tmp; cd tmp; echo .*/ | grep -F "../ ./"; cd ..; rmdir tmp')
[ "$res" == '' ] || err $LINENO

# command expansion

res=$($com <<< 'echo a$(seq 2)b')
[ "$res" == "a1 2b" ] || err $LINENO

res=$($com <<< 'echo a$()b')
[ "$res" == "ab" ] || err $LINENO

res=$($com <<< 'echo "a$(seq 2)b"')
[ "$res" == "a1
2b" ] || err $LINENO

res=$($com <<< 'echo $(pwd)')
[ "$res" == "$(pwd)" ] || err $LINENO

res=$($com <<< 'echo $(pwd) a')
[ "$res" == "$(pwd) a" ] || err $LINENO

res=$($com <<< 'echo {,,}$(date "+%w")')
[ "$res" == "$(echo {,,}$(date "+%w"))" ] || err $LINENO

res=$($com <<< 'echo $(date) | grep "  "')
[ "$?" == "1" ] || err $LINENO

# array

res=$($com <<< 'A=( a b ); echo ${A[1]}')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'A=( a b ); echo ${A[5 -4 ]}')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'A=( a b ); B=1; echo ${A[$B]}')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'A=( a b ); echo ${A[@]}')
[ "$res" == "a b" ] || err $LINENO

res=$($com <<< 'A=( a b ); A[0]=c ; echo ${A[@]}')
[ "$res" == "c b" ] || err $LINENO

res=$($com <<< 'A=( a b ); A[0]=( 1 2 )')
[ "$?" == 1 ] || err $LINENO

res=$($com <<< 'A=( a b ); A[]=1')
[ "$?" == 1 ] || err $LINENO


# symbol

res=$($com <<< 'echo ]')
[ "$res" == "]" ] || err $LINENO

# ansi-c quoting

res=$($com <<- FIN
echo $'aaa'
FIN
)
[ "$res" == "aaa" ] || err $LINENO

res=$($com <<- FIN
echo $'a\nb'
FIN
)
[ "$res" == "a
b" ] || err $LINENO

res=$($com <<- FIN
echo $'\c2\cr\cR\c-\c[\c]\c^\c<'
FIN
)
[ "$res" == $'\c2\cr\cR\c-\c[\c]\c^\c<' ] || err $LINENO

res=$($com <<- FIN
echo $'\110\19\9\477\x40\x7A\x7a\x9Z' 
FIN
)
[ "$res" == $'\110\19\9\477\x40\x7A\x7a\x9Z' ] || err $LINENO
#MEMO 128-255 is not the same with Bash

echo $0 >> ./ok
