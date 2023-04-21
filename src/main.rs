#![feature(slice_as_chunks)]
#![feature(allocator_api)]
#![feature(int_roundings)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![feature(pointer_byte_offsets)]
#![feature(option_result_contains)]
#![feature(pointer_is_aligned)]
#![no_std]
#![no_main]
#![allow(unused_imports)]

extern crate alloc;

mod allocator;
mod kalloc;
mod physical_memory_manager;
mod sbi_print;
mod start;
mod trap;
mod vm;

use crate::kalloc::PAGE_SIZE;
use crate::physical_memory_manager::MyMemoryRegion;
use crate::vm::KERNEL_PAGE_TABLE;
use alloc::string::String;
use core::arch::asm;
use core::panic::PanicInfo;

const OS_STACK_SIZE: usize = 65536; // Must be the same as in entry.S

#[repr(C, align(16))]
struct Stack([u8; OS_STACK_SIZE]);

#[no_mangle]
static STACK0: Stack = Stack([0; OS_STACK_SIZE]);

#[no_mangle]
fn main(_hart_id: usize, dtb: usize) -> ! {
    println!("---------- Kernel Start ----------");

    println!("> Setup kernel trap");
    unsafe {
        trap::setup_trap();
    }

    // DTB THING
    println!("Init Fdt Header");
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8).unwrap() };

    let free_memory_region = physical_memory_manager::get_free_memory(&fdt);
    unsafe {
        kalloc::init_page_allocator(free_memory_region);
    }
    vm::init_paging(&fdt);
    allocator::init_heap();

    let test = alloc::string::String::from("Hello World !");
    println!("{}", test);
    drop(test);

    unsafe {
        trap::enable_timer(&fdt);
    }

    println!("---------- Kernel End ----------");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[PANIC]: {:?}", info);
    loop {}
}
