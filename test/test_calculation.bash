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

# arithmetic calculation

res=$($com <<< 'echo $((12345 ))aaa')
[ "$res" == "12345aaa" ] || err $LINENO

res=$($com <<< 'echo $((echo 123 ) )')
[ "$res" == "123" ] || err $LINENO

res=$($com <<< 'echo $((
123
))')
[ "$res" == "123" ] || err $LINENO

res=$($com <<< 'echo $((
123
)
)')
[ "$?" == "0" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $((-\
123))')
[ "$res" == "-123" ] || err $LINENO

res=$($com <<< 'echo $((-\
12\
3))')
[ "$res" == "-123" ] || err $LINENO

res=$($com <<< 'echo $((123 ) ))')
[ "$?" == "2" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $((123 + 456))')
[ "$res" == "579" ] || err $LINENO

res=$($com <<< 'echo $((123 +456))')
[ "$res" == "579" ] || err $LINENO

res=$($com <<< 'echo $((123 + 456 + 1))')
[ "$res" == "580" ] || err $LINENO

res=$($com <<< 'echo $((123 + +456))')
[ "$res" == "579" ] || err $LINENO

res=$($com <<< 'echo $((456 + -123))')
[ "$res" == "333" ] || err $LINENO

res=$($com <<< 'echo $((456 -123))')
[ "$res" == "333" ] || err $LINENO

res=$($com <<< 'echo $((- - - 1))')
[ "$res" == "-1" ] || err $LINENO

res=$($com <<< 'echo $((- + - 1))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $((- (1 + 2 )))')
[ "$res" == "-3" ] || err $LINENO

res=$($com <<< 'echo $(( (1 + 2 ) ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'echo $(( (3) ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'echo $(( 1 + 2 * 3 ))')
[ "$res" == "7" ] || err $LINENO

res=$($com <<< 'echo $(( 1 + 2 / 3 ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( (1 + 2) / 3 ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( (1 + 2) / -3 ))')
[ "$res" == "-1" ] || err $LINENO

res=$($com <<< 'echo $(( (1 + 2) / - ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $(( ))')
[ "$res" == "0" ] || err $LINENO

res=$($com <<< 'echo $(( ( ) ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $((A ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( $A ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( $$ - 1 ))')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo $((A ))')
[ "$res" == "0" ] || err $LINENO

res=$($com <<< 'echo $((A + 3 ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'A=X; echo $((A ))')
[ "$res" == "0" ] || err $LINENO

res=$($com <<< 'A=X; echo $(( ++A ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'A=X; X=3 ; echo $(( ++A )); echo $A')
[ "$res" == "4
4" ] || err $LINENO

res=$(echo "echo \$(( '' ))" | $com)
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$(echo "echo \$(( '1' ))" | $com)
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$(echo "echo \$(( 1 '+' 1 ))" | $com)
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$(echo "echo \$(( 2 ** 10 ))" | $com)
[ "$res" == "1024" ] || err $LINENO

res=$(echo "echo \$(( 10000 ** 0 ))" | $com)
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 1 ** -1  ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1; echo $((A++ )); echo $A')
[ "$res" == "1
2" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("A"++ )); echo $A')
[ "$res" == "1
2" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("A "++ )); echo $A')
[ "$res" == "1
2" ] || err $LINENO

res=$($com <<< 'A=1; echo $((" A"++ )); echo $A')
[ "$res" == "1
2" ] || err $LINENO

res=$($com <<< 'A=1; echo $((++"A" )); echo $A')
[ "$res" == "2
2" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("A"-- )); echo $A')
[ "$res" == "1
0" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("A"--1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=10; echo $(( ++"$A" )) ; echo $A')
[ "$res" == "10
10" ] || err $LINENO

res=$($com <<< 'A=10; echo $(( "$A"++ ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("A"\
--1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=あああ; echo $((A++ ))')
[ "$?" == "1" ] || err $LINENO

#res=$($com <<< 'A=あああ; echo $((A++ )); echo $A')
#[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=5; echo $((A-- )); echo $A')
[ "$res" == "5
4" ] || err $LINENO

res=$($com <<< 'A=1; echo $((A++1 ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1; echo $((A ++1 ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1; echo $((A + +1 ))')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'A=2; echo $((A+-1 ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $((2++1 ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'A=1; echo $((2--1 ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'echo $(( -- ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( 1 ++ A ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( 1 ++ A )); echo $A')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( 1 ++ A ))
echo $A')
[ "$res" == "1" ] || err

res=$($com <<< 'A=1; echo $(("2""1"++1 ))')
[ "$res" == "22" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("2"++1 ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'A=1; echo $((++2++1 ))')
[ "$res" == "3" ] || err $LINENO

res=$($com <<< 'A=10 ; echo $(( ++A)); echo $A')
[ "$res" == "11
11" ] || err $LINENO

res=$($com <<< 'A=10 ; echo $(( ++"A")); echo $A')
[ "$res" == "11
11" ] || err $LINENO

res=$($com <<< 'A=B ; echo $(( ++$A)); echo $A')
[ "$res" == "1
B" ] || err $LINENO

res=$($com <<< 'A=10 ; echo $(( -- A)); echo $A')
[ "$res" == "9
9" ] || err $LINENO

res=$($com <<< 'A=10 ; echo $(( - - A)); echo $A')
[ "$res" == "10
10" ] || err $LINENO

res=$($com <<< 'A=10 ; echo $(( ++A++))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( === ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $(($$ / $$ ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( ! 123 )) $(( ! 0 ))')
[ "$res" == "0 1" ] || err $LINENO

res=$($com <<< 'echo $(( ~ 0 )) $(( ~ 1 )) $(( ~ -1 ))')
[ "$res" == "-1 -2 0" ] || err $LINENO

res=$($com <<< 'echo $(( 10 %3 )) $(( 10 %-3 )) $(( $$ % 1 ))')
[ "$res" == "1 1 0" ] || err $LINENO

res=$($com <<< 'echo $(( 10 % 0 ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $(( 1 << 1 )) $(( 1 << 10 )) $(( 1024 >> 2 ))')
[ "$res" == "2 1024 256" ] || err $LINENO

res=$($com <<< 'echo $(( 1 << -1 )) $(( 1 << -10 )) $(( 1024 >> -2 ))')
[ "$res" == "0 0 0" ] || err $LINENO

res=$($com <<< 'echo $(( 1 <= -1 )) $(( 1 >= -10 )) $(( 1024 > -2 )) $(( 1 < 3 ))')
[ "$res" == "0 1 1 1" ] || err $LINENO

res=$($com <<< 'echo $(( 1 <= 1 )) $(( 1 >= 1 )) $(( 1 > 1 )) $(( 1 < 1 ))')
[ "$res" == "1 1 0 0" ] || err $LINENO

res=$($com <<< 'echo $(( 3*1 <= 2 )) $(( 1 >= 1+4 ))')
[ "$res" == "0 0" ] || err $LINENO

res=$($com <<< 'echo $(( 1+1 == 2 )) $(( 1+1 != 2*1 ))')
[ "$res" == "1 0" ] || err $LINENO

res=$($com <<< 'echo $(( 1+1 & 2 )) $(( 1 ^ 2 )) $(( 3 ^ 1 )) $(( 1 & 2 )) $(( 2 | 1 )) ')
[ "$res" == "2 3 2 0 3" ] || err $LINENO

res=$($com <<< 'echo $((1+1&2)) $((1^2)) $((3^1)) $((1&2)) $((2|1)) ')
[ "$res" == "2 3 2 0 3" ] || err $LINENO

res=$($com <<< 'echo $(( 1+1 & -1 )) $(( 1 ^ -2 )) $(( 1 & -2 ))')
[ "$res" == "2 -1 0" ] || err $LINENO

res=$($com <<< 'echo $((123 && -1 )) $(( 0 && 10 )) $(( 0 || -1 || 0 )) $(( 0 || 0  )) $(( 0 && 0))')
[ "$res" == "1 0 1 0 0" ] || err $LINENO

res=$($com<<<'echo $((123&&-1))$((0&&10))$((0||-1||0))$((0||0))$((0&&0))')
[ "$res" == "10100" ] || err $LINENO

res=$($com<<<'echo $((A=1 && B=1))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com<<<'echo $((A=1 && (B=1) ))')
[ "$?" == "0" ] || err $LINENO
[ "$res" == "1" ] || err $LINENO

res=$($com<<<'echo $((A=1 && (B=1, 0) ))')
[ "$?" == "0" ] || err $LINENO
[ "$res" == "0" ] || err $LINENO

res=$($com<<<'echo $((A = 3 || 0 )); echo $A')
[ "$res" == "1
1" ] || err $LINENO

res=$($com<<<'echo $((A = 3 && 0 )); echo $A')
[ "$res" == "0
0" ] || err $LINENO

res=$($com<<<'echo $((A = 3 && 2 )); echo $A')
[ "$res" == "1
1" ] || err $LINENO

res=$($com <<< 'echo $(( 1? 20 : 30  )) $(( -5 + 5 ? 100 :  200))')
[ "$res" == "20 200" ] || err $LINENO

res=$($com <<< 'echo $(( (1? 20 : 30 ) + 3 )) $(( -5 + ( 5 ? 100 :  200)))')
[ "$res" == "23 95" ] || err $LINENO

res=$($com <<< 'echo $(( -(0? 20 : 30 ) * 3 )) $(( -5 + ( 5 ? 100 :  200)/5 ))')
[ "$res" == "-90 15" ] || err $LINENO

res=$($com <<< 'echo $(( A= 10 ))')
[ "$res" == "10" ] || err $LINENO

res=$($com <<< 'A=1 ; echo $(( A += 10 ))')
[ "$res" == "11" ] || err $LINENO

res=$($com <<< 'A=1 ; echo $(( A -= 10 ))')
[ "$res" == "-9" ] || err $LINENO

res=$($com <<< 'A=1 ; echo $(( A -= 10 + 2 )) $((A-=10+2))')
[ "$res" == "-11 -23" ] || err $LINENO

res=$($com <<< 'A=2 ; echo $(( A *= 10 + 2 )) $((A*=10+2))') 
[ "$res" == "24 288" ] || err $LINENO

res=$($com <<< 'A=-100 ; echo $(( A /= 10 + 2 )) $((A/=10+2))')
[ "$res" == "-8 0" ] || err $LINENO

res=$($com <<< 'A=-100 ; echo $(( A %= 10 + 2 )) $((A%=10+2))')
[ "$res" == "-4 -4" ] || err $LINENO

res=$($com <<< 'A=2 ; echo $(( A <<= 2 )) $((A<<=2)) $(( A <<= -1 ))')
[ "$res" == "8 32 0" ] || err $LINENO

res=$($com <<< 'A=-8 ; echo $(( A >>= 2 )) $((A>>=1)) $(( A >>= -1 ))')
[ "$res" == "-2 -1 0" ] || err $LINENO

res=$($com <<< 'A=-8 ; echo $((A^=2)) $((A&=1)) $((A|=-1))')
[ "$res" == "-6 0 -1" ] || err $LINENO

res=$($com <<< 'echo $((A=-8, A^=2)) $((A=3,A&=1)) $((A=9 ,A|=-1))')
[ "$res" == "-6 1 -1" ] || err $LINENO

res=$($com <<< 'echo $(( -" 12" )) $(( - "- 14" ))')
[ "$res" == "-12 14" ] || err $LINENO

res=$($com <<< 'echo $(( -"1 2" ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'echo $(( 0x11 )) $(( -"0x11" )) $(( - "- 0x11" ))')
[ "$res" == "17 -17 17" ] || err $LINENO

res=$($com <<< 'A=0x11; echo $(( A ))')
[ "$res" == "17" ] || err $LINENO

res=$($com <<< 'echo $(( -"011" )) $(( - "- 011" ))')
[ "$res" == "-9 9" ] || err $LINENO

res=$($com <<< 'echo $(( -"2#011" )) $(( - "- 2#0111101" ))')
[ "$res" == "-3 61" ] || err $LINENO

res=$($com <<< 'echo $(( 64#a )) $(( 64#A ))')
[ "$res" == "10 36" ] || err $LINENO

res=$($com <<< 'echo $(( 0xA )) $(( 0Xa ))')
[ "$res" == "10 10" ] || err $LINENO

res=$($com <<< 'echo $(( 17#A )) $(( 17#a ))')
[ "$res" == "10 10" ] || err $LINENO

res=$($com <<< 'echo $(( 37#A )) $(( 37#a ))')
[ "$res" == "36 10" ] || err $LINENO

res=$($com <<< 'echo $(( 64#@ )) $(( 64#_ ))')
[ "$res" == "62 63" ] || err $LINENO

res=$($com <<< 'echo $(( 62#@ ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 65#0 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(([#2] 1023)) $(( [#64]1023 )) $(([##64] 65279)) $(([#8] 64))')
[ "$res" == "2#1111111111 64#f_ fX_ 8#100" ] || err $LINENO

## float number calculation (sush original)

res=$($com <<< 'echo $((12345.0 ))aaa')
[ "$res" == "12345aaa" ] || err $LINENO

res=$($com <<< 'echo $((12345.01 ))aaa')
[ "$res" == "12345.01aaa" ] || err $LINENO

res=$($com <<< 'echo $((123.0 + 456.0))')
[ "$res" == "579" ] || err $LINENO

res=$($com <<< 'echo $((123 +456.0))')
[ "$res" == "579" ] || err $LINENO

res=$($com <<< 'echo $((123 + 456 + 1.1))')
[ "$res" == "580.1" ] || err $LINENO

res=$($com <<< 'echo $((123 + +456.2))')
[ "$res" == "579.2" ] || err $LINENO

res=$($com <<< 'echo $((456 + -123.9))')
[ "$res" == "332.1" ] || err $LINENO

res=$($com <<< 'echo $((- - - 1.09))')
[ "$res" == "-1.09" ] || err $LINENO

res=$($com <<< 'echo $((- (1 + 2.1 )))')
[ "$res" == "-3.1" ] || err $LINENO

res=$($com <<< 'echo $(( 1 + 2 * 3.2 ))')
[ "$res" == "7.4" ] || err $LINENO

res=$($com <<< 'echo $(( 1 + 2.0 / 3 )) $(( 1 + 2 / 3.0 ))')
[ "$res" == "1.6666666666666665 1.6666666666666665" ] || err $LINENO

res=$($com <<< 'echo $(( (1 + 2.0) / 3 ))')
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'A=1.23; echo $((A ))')
[ "$res" == "1.23" ] || err $LINENO

res=$($com <<< 'A=1.34; echo $(( $A ))')
[ "$res" == "1.34" ] || err $LINENO

res=$($com <<< 'echo $(( $$ - 1.1 ))')
[ "$?" == "0" ] || err $LINENO

res=$($com <<< 'echo $((A + 3.1 ))')
[ "$res" == "3.1" ] || err $LINENO

res=$($com <<< 'A=X; X=3.1 ; echo $(( ++A )); echo $A')
[ "$res" == "4.1
4.1" ] || err $LINENO

res=$(echo "echo \$(( 2.1 ** 10 ))" | $com)
[ "$res" == "1667.9880978201006" ] || err $LINENO

res=$(echo "echo \$(( 2.1 ** 3.3 ))" | $com)
[ "$res" == "11.569741950241465" ] || err $LINENO

res=$(echo "echo \$(( 1.23 ** 0 ))" | $com)
[ "$res" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 1.23 ** -1.1  ))')
[ "$?" == "1" ] || err $LINENO
[ "$res" == "" ] || err $LINENO

res=$($com <<< 'A=1.2; echo $((A++ )); echo $A')
[ "$res" == "1.2
2.2" ] || err $LINENO

res=$($com <<< 'A=1.3; echo $(("A"++ )); echo $A')
[ "$res" == "1.3
2.3" ] || err $LINENO

res=$($com <<< 'A=1.1; echo $((++"A" )); echo $A')
[ "$res" == "2.1
2.1" ] || err $LINENO

res=$($com <<< 'A=1.9; echo $(("A"-- )); echo $A')
[ "$res" == "1.9
0.8999999999999999" ] || err $LINENO

res=$($com <<< 'A=10.1; echo $(( ++"$A" )) ; echo $A')
[ "$res" == "10.1
10.1" ] || err $LINENO

res=$($com <<< 'A=5.1; echo $((A-- )); echo $A')
[ "$res" == "5.1
4.1" ] || err $LINENO

res=$($com <<< 'A=1.1; echo $((A + +1 ))')
[ "$res" == "2.1" ] || err $LINENO

res=$($com <<< 'A=2.2; echo $((A+-1 ))')
[ "$res" == "1.2000000000000002" ] || err $LINENO

res=$($com <<< 'A=1; echo $((2++1.9 ))')
[ "$res" == "3.9" ] || err $LINENO

res=$($com <<< 'A=1; echo $(("2""1"".9"++1 ))')
[ "$res" == "22.9" ] || err $LINENO

res=$($com <<< 'A=1; echo $((++2++1.2 ))')
[ "$res" == "3.2" ] || err $LINENO

res=$($com <<< 'echo $(( ! 123.1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( ~ 0.2 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 10.1 %3 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 10 %3.1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 1.1 << 1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 1 << 1.1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( 1.1 <= -1.1 )) $(( 1.1 >= -10.1 )) $(( 1024 > -2.2 )) $(( 1 < 3.2 ))')
[ "$res" == "0 1 1 1" ] || err $LINENO

res=$($com <<< 'echo $(( 1.01 <= 1.01 )) $(( 1.01 >= 1.01 )) $(( 1.01 > 1.01 )) $(( 1.01 < 1.01 ))')
[ "$res" == "1 1 0 0" ] || err $LINENO

res=$($com <<< 'echo $(( 3*1.1 <= 3.2 )) $(( 1.1 >= 1+4 ))')
[ "$res" == "0 0" ] || err $LINENO

res=$($com <<< 'echo $(( 1+1.1 == 2.1 )) $(( 1.1+1.1 != 2*1.1 ))')
[ "$res" == "1 0" ] || err $LINENO

res=$($com <<< 'echo $(( 1+1 & 2.1 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $((123 && -1.2 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'B=3; echo $(( A=1 || (B=1) )); echo $B')
[ "$res" == "1
3" ] || err $LINENO

res=$($com <<< 'B=3; echo $(( A=1 && (B=1) )); echo $B')
[ "$res" == "1
1" ] || err $LINENO

res=$($com <<< 'B=3; echo $(( A=1 && (B=1) || (B=4) )); echo $B')
[ "$res" == "1
1" ] || err $LINENO

res=$($com <<< 'B=3; echo $(( A=1 && (B=1, 0) || (B=4) )); echo $B')
[ "$res" == "1
4" ] || err $LINENO

res=$($com <<< 'echo $(( 1.0 ? 20 : 30  ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( -(0? 20 : 30.3 ) * 3 )) $(( -5 + ( 5 ? 100.5 :  200)/5 ))')
[ "$res" == "-90.9 15.100000000000001" ] || err $LINENO

res=$($com <<< 'echo $(( A= 10.1 ))')
[ "$res" == "10.1" ] || err $LINENO

res=$($com <<< 'A=1.1 ; echo $(( A += 10 ))')
[ "$res" == "11.1" ] || err $LINENO

res=$($com <<< 'A=1 ; echo $(( A -= 10.1 ))')
[ "$res" == "-9.1" ] || err $LINENO

res=$($com <<< 'A=1.1 ; echo $(( A -= 10 + 2 )) $((A-=10+2))')
[ "$res" == "-10.9 -22.9" ] || err $LINENO

res=$($com <<< 'A=2.2 ; echo $(( A *= 10 + 2 )) $((A*=10+2))') 
[ "$res" == "26.400000000000002 316.8" ] || err $LINENO

res=$($com <<< 'A=-100.2 ; echo $(( A /= 10 + 2 )) $((A/=10+2))')
[ "$res" == "-8.35 -0.6958333333333333" ] || err $LINENO

res=$($com <<< 'A=-100.2 ; echo $(( A %= 10 + 2 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=2.2 ; echo $(( A <<= 2 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'A=-8.1 ; echo $((A^=2))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( -" 12.3" )) $(( - "- 14.4" ))')
[ "$res" == "-12.3 14.4" ] || err $LINENO

res=$($com <<< 'echo $(( 0x11.2 ))')
[ "$?" == "1" ] || err $LINENO

res=$($com <<< 'echo $(( -" .3" )) $(( - "- .4" ))')
[ "$res" == "-0.3 0.4" ] || err $LINENO

res=$($com <<< 'echo $(( "1 + 1" ))')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'A=1; echo $(( "1 + A" * 3 ))')
[ "$res" == "4" ] || err $LINENO

res=$($com <<< 'echo $(( "1 << 1" ))')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'echo $(( "1 +" 1 ))')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'echo $(( 1 "+" 1 ))')
[ "$res" == "2" ] || err $LINENO

# use of array

res=$($com <<< 'echo $(( A[0] + 2 ))')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'echo $(( A[0] + 2 )); echo ${A[@]}')
[ "$res" == "2" ] || err $LINENO

res=$($com <<< 'A=(7 8 ) ; echo $(( A[1] + 2 ))')
[ "$res" == "10" ] || err $LINENO

res=$($com <<< 'echo $((++A[0])); echo ${A[@]}')
[ "$res" == "1
1" ] || err $LINENO

res=$($com <<< 'echo $((A[0]++)); echo ${A[@]}')
[ "$res" == "0
1" ] || err $LINENO

res=$($com <<< 'if ((0 >= 1 || 1 >= 1 )); then echo a ; else echo b ; fi')
[ "$res" == "a" ] || err $LINENO

res=$($com <<< 'if ((0 >= 1 && 1 >= 1 )); then echo a ; else echo b ; fi')
[ "$res" == "b" ] || err $LINENO

res=$($com <<< 'echo $(( 1 > 0 || 2 > 2 ))')
[ "$res" == "1" ] || err $LINENO

echo $0 >> ./ok
