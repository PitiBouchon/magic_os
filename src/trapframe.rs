// The TRAPFRAME must be align on a PAGE_SIZE because it should be placed in a PAGE (before the TRAMPOLINE)
#[repr(C, align(4096))]
#[derive(Debug)]
pub struct TrapFrame {
    pub kernel_satp: u64,   //   0 kernel page table
    pub kernel_sp: u64,     //   8 top of process's kernel stack
    pub kernel_trap: u64,   //  16 usertrap()
    pub epc: u64,           //  24 saved user program counter
    pub kernel_hartid: u64, //  32 saved kernel tp
    pub ra: u64,            //  40
    pub sp: u64,            //  48
    pub gp: u64,            //  56
    pub tp: u64,            //  64
    pub t0: u64,            //  72
    pub t1: u64,            //  80
    pub t2: u64,            //  88
    pub s0: u64,            //  96
    pub s1: u64,            // 104
    pub a0: u64,            // 112
    pub a1: u64,            // 120
    pub a2: u64,            // 128
    pub a3: u64,            // 136
    pub a4: u64,            // 144
    pub a5: u64,            // 152
    pub a6: u64,            // 160
    pub a7: u64,            // 168
    pub s2: u64,            // 176
    pub s3: u64,            // 184
    pub s4: u64,            // 192
    pub s5: u64,            // 200
    pub s6: u64,            // 208
    pub s7: u64,            // 216
    pub s8: u64,            // 224
    pub s9: u64,            // 232
    pub s10: u64,           // 240
    pub s11: u64,           // 248
    pub t3: u64,            // 256
    pub t4: u64,            // 264
    pub t5: u64,            // 272
    pub t6: u64,            // 280
}

impl TrapFrame {
    pub const fn new() -> Self {
        Self {
            kernel_satp: 0,
            kernel_sp: 0,
            kernel_trap: 0,
            epc: 0,
            kernel_hartid: 0,
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
        }
    }
}
