use crate::sbi_print::sbi_println_str;
use riscv::register::scause::Scause;
use riscv::register::sstatus::Sstatus;

#[no_mangle]
fn kernel_trap() {
    let _sepc: usize = riscv::register::sepc::read();
    let _sstatus: Sstatus = riscv::register::sstatus::read();
    let _scause: Scause = riscv::register::scause::read();
    let _spp = _sstatus.spp();
    sbi_println_str("Kernel Trap");
    loop {}
}
