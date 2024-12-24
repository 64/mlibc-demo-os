.section .text
.global trap_handler
.extern rust_trap_handler
.align 4
trap_handler:
    csrrw t6, sscratch, t6
    addi t6, t6, 256

    sd sp, 0(t6)
    mv sp, t6

    sd a0, 8(sp)
    sd a1, 16(sp)
    sd a2, 24(sp)
    sd a3, 32(sp)
    sd a4, 40(sp)
    sd a5, 48(sp)
    sd a6, 56(sp)
    sd a7, 64(sp)
    sd t0, 72(sp)
    sd t1, 80(sp)
    sd t2, 88(sp)
    sd t3, 96(sp)
    sd t4, 104(sp)
    sd t5, 112(sp)
    sd s0, 120(sp)
    sd s1, 128(sp)
    sd s2, 136(sp)
    sd s3, 144(sp)
    sd s4, 152(sp)
    sd s5, 160(sp)
    sd s6, 168(sp)
    sd s7, 176(sp)
    sd s8, 184(sp)
    sd s9, 192(sp)
    sd s10, 200(sp)
    sd s11, 208(sp)
    sd ra, 216(sp)
    sd tp, 224(sp)
    sd gp, 232(sp)

    addi t6, t6, -256
    csrrw t6, sscratch, t6
    sd t6, 240(sp)

    mv a0, sp
    call rust_trap_handler

    ld t6, 240(sp)

    ld gp, 232(sp)
    ld tp, 224(sp)
    ld ra, 216(sp)
    ld s11, 208(sp)
    ld s10, 200(sp)
    ld s9, 192(sp)
    ld s8, 184(sp)
    ld s7, 176(sp)
    ld s6, 168(sp)
    ld s5, 160(sp)
    ld s4, 152(sp)
    ld s3, 144(sp)
    ld s2, 136(sp)
    ld s1, 128(sp)
    ld s0, 120(sp)
    ld t5, 112(sp)
    ld t4, 104(sp)
    ld t3, 96(sp)
    ld t2, 88(sp)
    ld t1, 80(sp)
    ld t0, 72(sp)
    ld a7, 64(sp)
    ld a6, 56(sp)
    ld a5, 48(sp)
    ld a4, 40(sp)
    ld a3, 32(sp)
    ld a2, 24(sp)
    ld a1, 16(sp)
    ld a0, 8(sp)

    ld sp, 0(sp)

    sret
