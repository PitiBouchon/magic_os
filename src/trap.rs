use crate::println;
use riscv::register::scause::{Exception, Interrupt, Scause, Trap};
use riscv::register::sstatus::{Sstatus, SPP};

core::arch::global_asm!(include_str!("asm/kernelvec.S"));

#[no_mangle]
fn kernel_trap() {
    let _sepc: usize = riscv::register::sepc::read();
    let sstatus: Sstatus = riscv::register::sstatus::read();
    let scause: Scause = riscv::register::scause::read();
    let spp = sstatus.spp();
    match spp {
        SPP::Supervisor => println!("Trap from Supervisor"),
        SPP::User => println!("Trap from User"),
    }

    println!(
        "Kernel Trap Code : {} | {}",
        scause.code(),
        scause.is_interrupt()
    );
    match scause.cause() {
        Trap::Interrupt(i) => println!("Interrupt: {:?}", i),
        Trap::Exception(e) => println!("Exception: {:?}", e),
    }
    loop {}
}
