OUTPUT_ARCH("riscv")
OUTPUT_FORMAT("elf64-littleriscv")
ENTRY(_start);

SECTIONS {
    . = 0x80000000;
    PROVIDE(__kernel_start_phys = .);

    . = 0x80100000;
    .text : ALIGN(16) {
        *(.text.start)
        *(.text .text.*)
    }

    .srodata : ALIGN(16) {
        *(.srodata .srodata.*)
    }

    .sdata : ALIGN(16) {
        *(.sdata .sdata.*)
    }

    PROVIDE(__global_pointer$ = .);
    PROVIDE(__bss_start = .);

    .sbss : ALIGN(16) {
        *(.sbss .sbss.*)
    }

    .bss : ALIGN(16) {
        *(.bss .bss.*)
    }

    PROVIDE(__bss_end = .);

    .data : ALIGN(16) {
        *(.data .data.*)
    }

    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
    }

    .eh_frame : ALIGN(16) {
        *(.eh_frame)
    }

    . = ALIGN(4k);
    PROVIDE(__kernel_end_phys = .);

    /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}

