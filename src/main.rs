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

extern crate alloc;

// mod allocator;
mod cpu;
mod kernel_trap;
mod proc;
mod scheduler;
mod start;
mod trapframe;
mod user_trap;
mod vm;

use crate::cpu::{init_cpus, read_tp, write_tp};
use crate::proc::Proc;
use crate::scheduler::SCHEDULER;
use alloc::vec;
use core::ops::Deref;
use core::panic::PanicInfo;
use spin::Once;
use page_alloc::physical_memory_manager;
use sbi_print::println;
use crate::vm::KERNEL_PAGE_TABLE;

const INITCODE: [u8; 32] = [
    0x13, 0x05, 0xd0, 0x00, 0x93, 0x05, 0x40, 0x01, 0x93, 0x08, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00,
    0x6f, 0x00, 0x00, 0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
];

const OS_STACK_SIZE: usize = 65536; // Must be the same as in entry.S

#[repr(C, align(16))]
struct Stack([u8; OS_STACK_SIZE]);

#[no_mangle]
static STACK0: Stack = Stack([0; OS_STACK_SIZE]);

pub static HART_ID: Once<usize> = Once::new();

#[no_mangle]
fn main(hart_id: usize, dtb: usize) -> ! {
    println!("---------- Kernel Start ----------");

    println!("> Set hart id {}", hart_id);
    write_tp(hart_id);
    println!("tp regiser: {}", read_tp());

    println!("> Setup kernel trap");
    unsafe {
        kernel_trap::setup_trap();
    }

    // Parse the DTB
    println!("Init Fdt Header");
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8).unwrap() };

    let free_memory_region = physical_memory_manager::get_free_memory(&fdt);
    unsafe {
        page_alloc::init_page_allocator(free_memory_region);
    }
    vm::init_paging(&fdt);
    allocator::init_heap(KERNEL_PAGE_TABLE.deref());
    // After that it is possible to allocate memory

    let test1 = alloc::string::String::from("Hello World !");
    // println!("{}", test1);

    let test = vec![0, 1];
    // println!("{:?}", test);
    drop(test);
    drop(test1);

    println!("> Init Cpus");
    init_cpus(&fdt);

    // unsafe {
    //     kernel_trap::enable_timer(&fdt);
    // }

    println!("---------- Kernel End ----------");

    let test_proc = Proc::init_user_proc(&INITCODE);
    SCHEDULER.add_proc(test_proc);
    println!("Scheduling..");
    SCHEDULER.schedule()

    // loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[PANIC]: {:?}", info);
    loop {}
}
