OUTPUT_ARCH( "riscv" )
ENTRY( _start )
MEMORY
{
    ram (wxa) : ORIGIN = 0x80000000, LENGTH = 128M
}
/*
Specify program headers 
*/
PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}
/*
organize memory by section
*/
SECTIONS
{
    /*
        idk if its just me, but when linking with rust lld risc v expects
        these metadata sections that don't have to be loaded into memory
    */
    .riscv.attributes 0 : { *(.riscv.attributes) }
    .rela.dyn 0 : { *(.rela.dyn) }
    .rela.plt 0 : { *(.rela.plt) }
    .symtab 0 : { *(.symtab) }
    .symtab_shndx 0 : { *(.symtab_shndx) }
    .shstrtab 0 : { *(.shstrtab) }
    .strtab 0 : { *(.strtab) } 

    .text : {
        PROVIDE(_text_start = .);
        *(.text.init) *(.text .text.*)
        PROVIDE(_text_end = .);
        /*
        want linker to put the .text section into RAM, and specifically set the load memory address (LMA) to the same place
        *./
    } >ram AT>ram :text
    PROVIDE(_global_pointer = .);
    /*
    setup a read-only data section. Note that the linker can't provide any actual memory protection, we will program
    the MMU later. we place this in the .text program header as well
    */
    .rodata : {
        PROVIDE(_rodata_start = .);
        *(.rodata .rodata.*)
        PROVIDE(_rodata_end = .);
    } >ram AT>ram :text
    .data : {
        /*
            align the .data section to 4096 bytes (the resolution of 4kib pages)
        */
        . = ALIGN(4096);
        PROVIDE(_data_start = .);
        /*
        sdata is basically data but used for short quick critical sections. we tell the linker to load both sections here
        */
        *(.sdata .sdata.*) *(.data .data.*)
        PROVIDE(_data_end = .);
    } >ram AT>ram :data

    .bss : {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*) *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } >ram AT>ram :bss
    /*
        have linker provide some helpful locations for determining where the kernel stack starts and where heap begins and ends
        note that _memory_start points to the address we set above in MEMORY { ram (wxa) ...}, so you can change the start of memory
        to a different address later if necessary
    */
    PROVIDE(_memory_start = ORIGIN(ram));
    /*
        the stack grows from higher memory to lower memory (bottom to top). our kernel stack starts at _bss_end, so we need to
        allocate some amount of memory. we allocate 524 KiB to the kernel stack, which should be more than enough to get things going
    */
    PROVIDE(_stack_start = _bss_end);
    PROVIDE(_stack_end = _stack_start + 0x80000);
    PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));
    /*
        we then make our heap pointers for use in defining our heap allocation later (though since set here can use for kernel too)
    */
    PROVIDE(_heap_start = _stack_end);
    PROVIDE(_heap_end = _memory_end - _heap_start);
}
