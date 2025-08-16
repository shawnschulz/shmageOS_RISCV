# asm trap handler

.section .text
.global asm_trap_vector
asm_trap_vector:
    /* if the cpu gets interrupted for any reason, we enter here (and therefore jump back to our rust code) */
    mret
