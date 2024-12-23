.section .text
.global ret_to_user
ret_to_user:
    mv sp, a0
    sret
