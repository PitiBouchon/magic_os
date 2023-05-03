use crate::cpu::get_cpu;
use crate::proc::{Proc, ProcContext, ProcState};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::DerefMut;
use spin::Mutex;
use sbi_print::println;

core::arch::global_asm!(include_str!("asm/switch.S"));

extern "Rust" {
    // store ctx1 and load ctx2
    fn switch(ctx1: *mut ProcContext, ctx2: *mut ProcContext);
}

pub static SCHEDULER: Scheduler = Scheduler::new();

pub struct Scheduler {
    used: Mutex<Vec<Proc>>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            used: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn add_proc(&self, proc: Proc) {
        let mut used_list = self.used.lock();
        used_list.push(proc);
    }

    pub fn schedule(&self) -> ! {
        loop {
            let mut used_list = self.used.lock();
            match used_list.pop() {
                Some(mut proc) => {
                    let mut cpu_guard = get_cpu();
                    let mut cpu = cpu_guard.deref_mut();
                    proc.state = ProcState::Running;
                    println!("Switching to proc: {}", proc.name);
                    cpu.proc = Some(Box::new(proc));
                    unsafe {
                        let scheduler_ctx = &mut cpu.scheduler_context as *mut ProcContext;
                        let proc_ctx = &mut cpu.proc.as_mut().unwrap().context as *mut ProcContext;
                        drop(cpu_guard);
                        switch(scheduler_ctx, proc_ctx)
                    }
                    let mut cpu_guard = get_cpu();
                    let cpu = cpu_guard.deref_mut();
                    let mut used_list = self.used.lock();
                    let proc = cpu.proc.take().unwrap();
                    used_list.push(*proc); // Could do `Box::<Proc>::into_inner(proc)` instead
                    drop(cpu_guard);
                }
                None => unsafe {
                    riscv::asm::wfi();
                },
            }
        }
    }
}
