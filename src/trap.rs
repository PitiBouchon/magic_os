use crate::println;
use bit_field::BitField;
use core::arch::asm;
use fdt::Fdt;
use riscv::register::scause::{Exception, Interrupt, Scause, Trap};
use riscv::register::sstatus::{Sstatus, SPP};
use riscv::register::stvec::TrapMode;

core::arch::global_asm!(include_str!("asm/kernelvec.S"));

extern "C" {
    pub fn kernelvec();
}

pub unsafe fn setup_trap() {
    riscv::register::stvec::write(kernelvec as usize, TrapMode::Direct);
}

pub unsafe fn enable_timer(fdt: &Fdt) {
    if fdt.cpus().all(|cpu| {
        if let Some(ext_prop) = cpu.property("riscv,isa") {
            if let Ok(isa) = core::str::from_utf8(ext_prop.value) {
                return isa.contains("sstc");
            }
        }
        false
    }) {
        SSTC_EXENTION = true;
    }
    timer_init();
}

unsafe fn write_stimecmp(time: u64) {
    asm!("csrw 0x14D, {time}", time = in(reg) time);
}

unsafe fn timer_init() {
    const SCHED_INTERVAL: u64 = 10_000_000;
    if SSTC_EXENTION {
        write_stimecmp(riscv::register::time::read64() + SCHED_INTERVAL);
    } else {
        sbi::timer::set_timer(riscv::register::time::read64() + SCHED_INTERVAL).unwrap();
    }
}

// TODO : check if Once could be used
static mut SSTC_EXENTION: bool = false;

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
        Trap::Interrupt(i) => {
            println!("Interrupt: {:?}", i);
            if i == Interrupt::SupervisorTimer {
                unsafe {
                    timer_init();
                }
            }
        }
        Trap::Exception(e) => println!("Exception: {:?}", e),
    }
}
