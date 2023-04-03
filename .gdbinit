set confirm off
set architecture riscv:rv64
target remote 127.0.0.1:26000
symbol-file target/riscv64i-custom_target/debug/magic_os.elf
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes
