#!/bin/sh
set -ex

MLIBC_DIR=$HOME/mlibc
CC_RUNTIME_DIR=$HOME/cc-runtime

OUT_DIR=target/riscv64imac-unknown-none-elf
mkdir -p $OUT_DIR

# Build + install mlibc.
# Note: do meson setup first.
DESTDIR=$MLIBC_DIR/install-headers ninja -C $MLIBC_DIR/build install

riscv64-elf-gcc \
    -static -nostdinc -nostdlib -g \
    -I $MLIBC_DIR/install-headers/include \
    -L $MLIBC_DIR/build \
    $MLIBC_DIR/install-headers/lib/crt1.o \
    user/user_test.c \
    $MLIBC_DIR/build/libc.a \
    $HOME/cc-runtime/cc-runtime.a \
    -o $OUT_DIR/user_test
