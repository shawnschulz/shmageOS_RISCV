# boot.S
# bootloader for SoS
# Stephen Marz
# 8 February 2019
.option norvc
.section .data
.section .text.init
.global _start
.global print_string
.global start_string
.global line_check
_start:
	li	a0, 0x23
	jal	ra, uart_put_char
	li	a0, 0x23
	jal	ra, uart_put_char
	li	a0, 0x23
	jal	ra, uart_put_char
	li	a0, 0x23
	jal	ra, uart_put_char
	li	a0, 0x0A
	jal	ra, uart_put_char

	la		sp, _stack_start

	la t0, kernel_main    # address of main
	csrw sepc, t0         # set S-mode exception PC to main

	csrr t1, sstatus
	ori t1, t1, 0x2       # set SIE bit (enable S-mode interrupts)
	csrw sstatus, t1

	sret

	# la a1, start_string
	# jal ra, print_string


	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
#	csrr	t0, mhartid
# 	li a7, 0          # SBI function ID: 0 = sbi_get_hartid
#	ecall             # trap into M-mode

	bnez	t0, 3f

	la a1, line_check
	jal ra, print_string

	# SATP should be zero, but let's make sure
#	csrw	satp, zero
.option push
.option norelax
	la		gp, _global_pointer
.option pop
	# The BSS section is expected to be zero
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	# Control registers, set the stack, mstatus, mepc,
	# and mtvec to return to the main function.
	# li		t5, 0xffff;
	# csrw	medeleg, t5
	# csrw	mideleg, t5
	la		sp, _stack_start
	# We use mret here so that the mstatus register
	# is properly updated.
	li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
#	csrw	mstatus, t0
	la		t1, kernel_main
	csrw	mepc, t1


	li a7, 0x2        # SBI call ID: sbi_set_mepc (implementation-dependent)
	la a0, kernel_main
	ecall

	la		t2, asm_trap_vector
	#csrw	mtvec, t2

	li a7, 0x3        # SBI call ID: sbi_set_mtvec (implementation-dependent)
	la a0, asm_trap_vector
	ecall

	li		t3, (1 << 3) | (1 << 7) | (1 << 11)
	# csrw	mie, t3
	csrs sstatus, 0x2     # set SIE bit in sstatus (S-mode interrupt enable)
	la		ra, 4f
	mret
3:

	# Parked harts go here. We need to set these
	# to only awaken if it receives a software interrupt,
	# which we're going to call the SIPI (Software Intra-Processor Interrupt).
	# We only use these to run user-space programs, although this may
	# change.
4:
	wfi
	j		4b

print_string:
    mv t1, a1          # t0 = pointer to string

1:  lb a2, 0(t0)       # load byte from string
	li a3, 0x0A
    jal ra, uart_put_char
    beq a2, a3, 2f   # if null terminator, done
    addi t1, t1, 1
    j 1b

2:  ret

uart_put_char:
	# very silly, only works with absolute address
	# 0xD4017000
	# 0x10000000
	li	t0, 0xD4017000
#	li	t0, 0x10000000
	sb	a0, 0(t0)
	ret

start_string:
    .asciz "----- Entering ShmageOS ----- \n"

line_check:
    .asciz "at line 32 \n"
