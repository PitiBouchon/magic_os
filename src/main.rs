#![no_std]
#![no_main]

#![allow(unused_variables)]
#![allow(unused_imports)]

mod start;
mod trap;
mod sbi_print;
mod dtb;

use core::panic::PanicInfo;
use riscv::register::stvec::TrapMode;
use sbi_print::sbi_println_str;
use crate::dtb::init_dtb;

const OS_STACK_SIZE: usize = 65536;

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
    sbi_println_str("Init Fdt Header");
    unsafe { init_dtb(dtb); }

    sbi_println_str("---------- Kernel End ----------");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sbi_println_str("[PANIC]");
    loop {}
}

// mod some_mod {
//     use crate::sbi_print::sbi_println_str;
//
//     pub struct A;
//
//     impl A {
//         pub fn check_print() {
//             sbi_println_str("Hi here");
//         }
//     }
// }