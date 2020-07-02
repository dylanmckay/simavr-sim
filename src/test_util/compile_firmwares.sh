#! /bin/sh

BASEDIR=$(dirname "$0")
cd $BASEDIR

avr-gcc -mmcu=atmega328 atmega328-factorial.cpp -o atmega328-factorial.elf -std=c++11

