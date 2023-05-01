OUTPUT_ARCH("riscv");
ENTRY(_entry);

SECTIONS
{
    . = 0x80200000; /* Must be the same as in build.rs */

    .text : {
        *(.text.entry);
        *(.text .text.*);
        . = ALIGN(0x1000);
        PROVIDE(_trampoline = .);
        *(trampsec)
        . = ALIGN(0x1000);
        ASSERT(. - _trampoline == 0x1000, "error: trampoline larger than one page");
        PROVIDE(_kernel_end_text = .);
    }

    .rodata : {
        . = ALIGN(16);
        *(.srodata .srodata.*) /* do not need to distinguish this from .rodata */
        . = ALIGN(16);
        *(.rodata .rodata.*)
    }

    .data : {
        . = ALIGN(16);
        PROVIDE( __global_pointer$ = . ); /* Should be placed at the middle of .sdata see : https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/ */
        *(.sdata .sdata.*) /* do not need to distinguish this from .data */
        . = ALIGN(16);
        *(.data .data.*)
    }

    .bss : {
        _start_bss = .;
        . = ALIGN(16);
        *(.sbss .sbss.*) /* do not need to distinguish this from .bss */
        . = ALIGN(16);
        *(.bss .bss.*)
        _end_bss = .;
    }

    PROVIDE(_kernel_end = .);
};
