.section .text
.global _start
_start:
    li a0, 123
    ecall
    li a0, 456
    ecall
.done:
    j .done

