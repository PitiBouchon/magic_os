#![no_std]
#![no_main]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate alloc;

mod allocator;
mod sbi_print;
mod start;
mod trap;

use alloc::string::String;
use core::panic::PanicInfo;
use riscv::register::stvec::TrapMode;

const OS_STACK_SIZE: usize = 65536;

core::arch::global_asm!(include_str!("asm/entry.S"));

#[repr(C, align(16))]
struct Stack([u8; OS_STACK_SIZE]);

#[no_mangle]
static STACK0: Stack = Stack([0; OS_STACK_SIZE]);

extern "C" {
    fn kernelvec();
}

#[no_mangle]
fn main(hart_id: usize, dtb: usize) -> ! {
    println!("---------- Kernel Start ----------");

    println!("> Setup kernel trap");
    unsafe {
        riscv::register::stvec::write(kernelvec as usize, TrapMode::Direct);
    }

    // DTB THING
    println!("Init Fdt Header");
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8).unwrap() };


    allocator::init_heap(&fdt).unwrap();

    println!("---------- Kernel End ----------");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[PANIC]");
    loop {}
}
