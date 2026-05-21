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

	la t0, s_trap # address of trap function in our rust kernel (must extern C this)
	csrw stvec, t0

	csrr t1, sstatus
	ori t1, t1, 0x2       # set SIE bit (enable S-mode interrupts)
	csrw sstatus, t1

	la		ra, 4f
	sret
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
