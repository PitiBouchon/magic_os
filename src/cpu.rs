use crate::proc::{Proc, ProcContext};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::asm;
use fdt::Fdt;
use spin::{Mutex, MutexGuard, Once};

// Here we are using the register tp (Thread Pointer) just as a storage variable (because there are currently no thread)
pub fn get_cpuid() -> usize {
    read_tp()
}

pub fn read_tp() -> usize {
    let tp: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) tp);
    }
    tp
}

pub fn write_tp(tp: usize) {
    unsafe {
        asm!("mv tp, {}", in(reg) tp);
    }
}

pub(crate) struct Cpu {
    pub proc: Option<Box<Proc>>,
    pub scheduler_context: ProcContext,
    // pub interrupt_base: Mutex<bool>,
    // pub push_count: Mutex<u32>,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            proc: None,
            scheduler_context: ProcContext {
                ra: 0,
                sp: 0,
                s: [0; 12],
            },
            // interrupt_base: Mutex::new(false),
            // push_count: Mutex::new(0),
        }
    }
}

static CPUS: Once<Vec<Mutex<Cpu>>> = Once::new();

pub fn init_cpus(fdt: &Fdt) {
    CPUS.call_once(|| fdt.cpus().map(|_| Mutex::new(Cpu::new())).collect());
}

pub(crate) fn get_cpu() -> MutexGuard<'static, Cpu> {
    let cpu_id = get_cpuid();
    let cpus = CPUS.get().unwrap();
    cpus.get(cpu_id).unwrap().lock()
}
