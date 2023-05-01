use crate::cpu::{get_cpu, get_cpuid};
use crate::println;
use crate::trapframe::TrapFrame;
use crate::vm::page_table::entry::addr::PhysicalAddr;
use crate::vm::{TRAMPOLINE, TRAPFRAME};
use bit_field::BitField;
use riscv::register::satp::Mode;
use riscv::register::scause::Exception::UserEnvCall;
use riscv::register::scause::Trap;
use riscv::register::sstatus::SPP;
use riscv::register::stvec::TrapMode;

extern "C" {
    fn uservec();
    fn userret();
    pub fn trampoline();
}

#[no_mangle]
pub unsafe fn usertrapret() {
    let mut cpu = get_cpu();
    let proc = cpu.proc.as_mut().unwrap();

    println!("UserTrapRet");

    riscv::register::stvec::write(
        *TRAMPOLINE.get() as usize + uservec as usize - trampoline as usize,
        TrapMode::Direct,
    );

    let mut trapframe: &mut TrapFrame = proc.trap_frame.as_mut();
    trapframe.kernel_satp = riscv::register::satp::read().bits() as u64;
    trapframe.kernel_sp = *proc.kernel_stack.get();
    trapframe.kernel_trap = usertrap as usize as u64;
    trapframe.kernel_hartid = get_cpuid() as u64;

    riscv::register::sstatus::set_spp(SPP::User);
    riscv::register::sstatus::set_spie();

    // TODO : Wtf is this ?
    riscv::register::sepc::write(trapframe.epc as usize);

    let mut satp = 0;
    satp.set_bits(60..64, Mode::Sv39 as usize as u64);
    satp.set_bits(44..60, 0); // ASID
    satp.set_bits(
        0..44,
        PhysicalAddr::new(proc.page_table.as_ref() as *const _ as u64)
            .ppn()
            .get(),
    ); // PPN
    let a = proc.page_table.as_ref() as *const _ as u64;
    assert!(a < 0x100000000000);

    let userret = *TRAMPOLINE.get() as usize + userret as usize - trampoline as usize;
    let fp = userret as *const ();
    let code: fn(u64, u64) = core::mem::transmute(fp);
    code(*TRAPFRAME.get(), satp)
}

// TODO : disable interrupt during a trap I guess
fn usertrap() {
    println!("USER TRAP");
    let scause = riscv::register::scause::read();
    match scause.cause() {
        Trap::Interrupt(i) => println!("Received interrupt: {:?}", i),
        Trap::Exception(e) => {
            println!("Received Exception: {:?}", e);
            if e == UserEnvCall {
                println!("Received a syscall !");
            }
        }
    }
}
