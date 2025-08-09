.option norvc 
.section .data
.section .text.init
.global _start
_start:
    # read t0 
    csrr    t0, mhartid
    # branch to 3 if t0 is not equal to 0
    bnez    t0, 3f
    # ensure SATP is zero by writing zero to the register
    csrw    satp, zero
# i think this means push the norelax option to the option stack?
.option push
.option norelax
    # compute the value in the _global_pointer address and store in the gp register
    la  gp, _global_pointer
# pop the norelax option from the stack
.option pop
    # Need to clear unitialized global and static memory in the bss section
    la  a0, _bss_start
    la  a1, _bss_end
    # Goto 2 if _bss_end is greater than _bss_start
    bgeu    a0, a1, 2f
1:
    # Store 0 in a0 to ensure bss memory is 0
    sd  zero, (a0)
    # Move to the next memory segment by incrementing the address by 8
    addi    a0, a0, 8
    # If that next segment isn't 0, also make that segment 0
    bltu    a0, a1, 1b
2:
3:
    # Loop waiting for an interrupt
    wfi
    j   3b

# allocate the stack pointer
la  sp, _stack
# set the 11th bit of t0 to 11, bits 7 and 3 to 1. This sets the status to 'machine mode'
# and enables coarse control of interrupts
li  t0, (0b11 << 11) | (1 << 7) | (1 << 3)
# write to the mstatus control status register
csrw    mstatus, t0
# load the pointer to what will be our kernel main process in rust
la  t1, kernel_main
# write the pointer to the machine excpetion program counter so our processor can start running our kernel
csrw    mepc, t1
la  t2, asm_trap_vector
# our kernel will eventulaly handle 'traps' (for syscalls, illegal asm instructions and timer interrupts),
# so we need to write the pointer to risc-v's mtvec location
csrw    mtvec, t2
# enable certain interrupts in the machine intterupt enable register
li  t3, (1 << 3) | (1 << 7) | (1 << 11)
csrw mie, t3
# jump back through the mepc register when we are done with kernel's main function
la  ra, 4f
mret
4: 
    wfi
    j   4b
