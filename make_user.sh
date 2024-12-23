#!/bin/sh
set -ex

riscv64-elf-as src/user_test.asm -o target/riscv64imac-unknown-none-elf/user_test.o
riscv64-elf-ld target/riscv64imac-unknown-none-elf/user_test.o -o target/riscv64imac-unknown-none-elf/user_test
