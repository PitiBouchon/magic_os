#![no_std]
#![no_main]

#![allow(unused_variables)]

mod start;
mod trap;
mod dtb;
mod sbi_print;

use core::panic::PanicInfo;
use riscv::register::stvec::TrapMode;
use crate::dtb::FdtHeader;
use crate::sbi_print::sbi_println_str;

const OS_STACK_SIZE: usize = 8192;

#[repr(C, align(16))]
struct Stack([u8; OS_STACK_SIZE]);

#[no_mangle]
static STACK0: Stack = Stack([0; OS_STACK_SIZE]);

extern "C" {
    fn kernelvec();
}

#[no_mangle]
fn main(hart_id: usize, dtb: usize) -> ! {
    sbi_println_str("---------- Kernel Start ----------");

    sbi_println_str("> Setup kernel trap");
    unsafe { riscv::register::stvec::write(kernelvec as usize, TrapMode::Vectored); }

    // DTB THING
    unsafe {
        FdtHeader::init_fdt_header(dtb);
    }

    sbi_println_str("---------- Kernel End ----------");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sbi_println_str("[PANIC]");
    loop {}
}
