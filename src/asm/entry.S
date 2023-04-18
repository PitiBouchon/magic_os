.pushsection .text.entry
.equ OS_STACK_SIZE, 65536 # Must be the same value as in main.rs
.global _entry
_entry:
	la sp, STACK0 # Must be the same name as in main.rs
	li t0, OS_STACK_SIZE # The stack is upside down
	add sp, sp, t0

	# jump to start() in start.rs
	call start

spin:
	j spin
.popsection
