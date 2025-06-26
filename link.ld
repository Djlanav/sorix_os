ENTRY(_start)

SECTIONS {
    /* Start at 20 MiB = 0x1400000 */
    . = 0x100000;

    /* TEXT: Code section */
    .text ALIGN(0x1000) : {
        KEEP(*(.text._start))  /* Keep your _start symbol if needed */
        *(.text .text.*)
    }

    /* RODATA: Read-only data, strings, consts */
    .rodata ALIGN(0x1000) : {
        *(.rodata .rodata.*)
    }

    /* DATA: Writable globals, statics */
    .data ALIGN(0x1000) : {
        *(.data .data.*)
    }

    /* GOT: Global offset table */
    .got ALIGN(0x1000) : {
        *(.got .got.*)
    }

    /* BSS: Zero-initialized statics */
    .bss ALIGN(0x1000) : {
        __bss_start = .;
        *(.bss .bss.* COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame)  /* Optional: drop Rust’s panic unwind junk if unused */
    }
}
