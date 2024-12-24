#!/bin/sh
set -ex

# riscv64-elf-as src/user_test.asm -o target/riscv64imac-unknown-none-elf/user_test.o
# riscv64-elf-ld target/riscv64imac-unknown-none-elf/user_test.o -o target/riscv64imac-unknown-none-elf/user_test

DESTDIR=$HOME/mlibc/install-headers ninja -C ~/mlibc/build install

riscv64-elf-gcc -nostdinc -nostdlib -I $HOME/mlibc/install-headers/include \
    src/user_test.c -g -c \
    -o target/riscv64imac-unknown-none-elf/user_test.o

riscv64-elf-gcc -static -nostdinc -nostdlib -g -L$HOME/mlibc/build \
    $HOME/mlibc/install-headers/lib/crt1.o \
    target/riscv64imac-unknown-none-elf/user_test.o \
    $HOME/mlibc/build/libc.a \
    $HOME/cc-runtime/cc-runtime.a \
    -o target/riscv64imac-unknown-none-elf/user_test
