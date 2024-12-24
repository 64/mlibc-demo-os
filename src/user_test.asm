.section .rodata
helloworld:
    .ascii "Hello World!\n"

.section .text
.global _start
_start:
    # Write
    li a0, 1
    li a1, 1
    la a2, helloworld
    li a3, 13
    ecall

    # Exit
    li a0, 0
    li a1, 123
    ecall
