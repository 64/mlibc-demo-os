.section .text
.global trap_handler
.extern rust_trap_handler
.align 4
trap_handler:
    csrrw t6, sscratch, t6
    addi t6, t6, 128

    sd sp, 0(t6)
    mv sp, t6

    sd a0, 8(sp)

    addi t6, t6, -128
    csrrw t6, sscratch, t6
    sd t6, 16(sp)

    mv a0, sp
    call rust_trap_handler

    ld t6, 16(sp)
    ld a0, 8(sp)
    ld sp, 0(sp)

    sret
