OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)


OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data   PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text ALIGN(4K) :
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    } :text

    . = ALIGN(4K);

    .limine_requests ALIGN(4K) :
    {
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests.*))
    } :rodata

    . = ALIGN(4K);

    .data :
    {
        *(.data .data.*)
    } :data

    .bss :
    {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
        *(.comment)
    }
}
