/*
 link_sbi.lds
 Linker script for outputting to RISC-V QEMU "virt" machine using opensbi as bios.
 Shawn Schulz
 5 December 2025
*/

OUTPUT_ARCH( "riscv" )

/*
*/
ENTRY( _start )

MEMORY
{
  DDRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS
{

  /*
    The first part of our DRAM layout will be the text section.
	Since our CPU instructions are here, and our memory starts at
	0x8000_0000, we need our entry point to line up here.
  */
  .text : {
	  /*
	    PROVIDE allows me to access a symbol called _text_start so
		I know where the text section starts in the operating system.
		This should not move, but it is here for convenience.
		The period '.' tells the linker to set _text_start to the
		CURRENT location ('.' = current memory location). This current
		memory location moves as we add things.
	  */

    PROVIDE(_text_start = .);
	/*
	  We are going to layout all text sections here, starting with
	  .text.init. The asterisk in front of the parentheses means to match
	  the .text.init section of ANY object file. Otherwise, we can specify
	  which object file should contain the .text.init section, for example,
	  boot.o(.text.init) would specifically put the .text.init section of
	  our bootloader here.

	  Because we might want to change the name of our files, we'll leave it
	  with a *.

	  Inside the parentheses is the name of the section. I created my own
	  called .text.init to make 100% sure that the _start is put right at the
	  beginning. The linker will lay this out in the order it receives it:

	  .text.init first
	  all .text sections next
	  any .text.* sections last

	  .text.* means to match anything after .text. If we didn't already specify
	  .text.init, this would've matched here. The assembler and linker can place
	  things in "special" text sections, so we match any we might come across here.
	*/
    *(.text.init) *(.text .text.*)
	/*
	  Again, with PROVIDE, we're providing a readable symbol called _text_end, which is
	  set to the memory address AFTER .text.init, .text, and .text.*'s have been added.
	*/
    PROVIDE(_text_end = .);
	/*
	  The portion after the right brace is in an odd format. However, this is telling the
	  linker what memory portion to put it in. We labeled our DRAM, DRAM, with the constraints
	  that it is writeable, allocatable, and executable. The linker will make sure with this
	  that we can do all of those things.

	  >DRAM - This just tells the linker script to put this entire section (.text) into the
	         DRAM region of memory. To my knowledge, the '>' does not mean "greater than". Instead,
			 it is a symbol to let the linker know we want to put this in DRAM.

	  AT>DRAM - This sets the LMA (load memory address) region to the same thing. LMA is the final
	           translation of a VMA (virtual memory address). With this linker script, we're loading
			   everything into its physical location. We'll let the kernel copy and sort out the
			   virtual memory. That's why >DRAM and AT>DRAM are continually the same thing.

	  :text  - This tells the linker script to put this into the :text progDRAM header. We've only
	           defined three: text, data, and bss. In this case, we're telling the linker script
			   to go into the text section.
	*/
  } >DRAM AT>DRAM :text
   /*
     The global pointer allows the linker to position global variables and constants into
	 independent positions relative to the gp (global pointer) register. The globals start
	 after the text sections and are only relevant to the rodata, data, and bss sections.
   */
   PROVIDE(_global_pointer = .);
   /*
     Most compilers create a rodata (read only data) section for global constants. However,
	 we're going to place ours in the text section. We can actually put this in :data, but
	 since the .text section is read-only, we can place it there.

	 NOTE: This doesn't actually do anything, yet. The actual "protection" cannot be done
	 at link time. Instead, when we progDRAM the memory management unit (MMU), we will be
	 able to choose which bits (R=read, W=write, X=execute) we want each memory segment
	 to be able to do.
   */
  .rodata : {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);
	/*
	   Again, we're placing the rodata section in the memory segment "DRAM" and we're putting
	   it in the :text progDRAM header. We don't have one for rodata anyway.
	*/
  } >DRAM AT>DRAM :text

  .data : {
	/*
	   . = ALIGN(4096) tells the linker to align the current memory location (which is
	   0x8000_0000 + text section + rodata section) to 4096 bytes. This is because our paging
	   system's resolution is 4,096 bytes or 4 KiB.
	*/
    . = ALIGN(4096);
    PROVIDE(_data_start = .);
	/*
	   sdata and data are essentially the same thing. However, compilers usually use the
	   sdata sections for shorter, quicker loading sections. So, usually critical data
	   is loaded there. However, we're loading all of this in one fell swoop.
	   So, we're looking to put all of the following sections under the umbrella .data:
	   .sdata
	   .sdata.[anything]
	   .data
	   .data.[anything]

	   ...in that order.
	*/
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  } >DRAM AT>DRAM :data

  .bss : {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  } >DRAM AT>DRAM :bss

  /*
     The following will be helpful when we allocate the kernel stack (_stack) and
	 determine where the heap begnis and ends (_heap_start and _heap_start + _heap_size)/
	 When we do memory allocation, we can use these symbols.

	 We use the symbols instead of hard-coding an address because this is a floating target.
	 As we add code, the heap moves farther down the memory and gets shorter.

	 _memory_start will be set to 0x8000_0000 here. We use ORIGIN(DRAM) so that it will take
	 whatever we set the origin of DRAM to. Otherwise, we'd have to change it more than once
	 if we ever stray away from 0x8000_0000 as our entry point.
  */
  PROVIDE(_memory_start = ORIGIN(DRAM));
}
