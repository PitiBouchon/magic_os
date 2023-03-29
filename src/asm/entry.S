.section .text.entry
.global _entry
_entry:
	la sp, STACK0

	# jump to start() in start.c
	call start

spin:
	j spin
