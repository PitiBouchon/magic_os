pub mod page_table;

use crate::kalloc::{page_round_up, PAGE_ALLOCATOR, PAGE_SIZE};
use crate::println;
use crate::trapframe::TrapFrame;
use crate::vm::page_table::addr::MAX_VIRTUAL_ADDR;
use crate::vm::page_table::entry::perm::PTEPermission;
use alloc::boxed::Box;
use core::ops::Deref;
use fdt::Fdt;
use page_table::addr::{PhysicalAddr, VirtualAddr};
use page_table::PageTable;
use riscv::register::satp::Mode;
use spin::{Lazy, Mutex};

pub const TRAMPOLINE: VirtualAddr = VirtualAddr::new(MAX_VIRTUAL_ADDR - PAGE_SIZE as u64);
pub const TRAPFRAME: VirtualAddr = TRAMPOLINE.sub_offset(PAGE_SIZE as u64);

extern "C" {
    static _kernel_end_text: u8;
    static _kernel_end: u8;
    static _trampoline: u8;
}

pub(crate) static KERNEL_PAGE_TABLE: Lazy<Mutex<&mut PageTable>> = Lazy::new(|| {
    let kernel_page_table: &mut PageTable =
        unsafe { &mut *(PAGE_ALLOCATOR.kalloc().unwrap().cast().as_ptr()) };
    Mutex::new(kernel_page_table)
});

pub fn init_paging(_fdt: &Fdt) {
    // TODO : Should use the Fdt to map things I guess
    println!("Setup Page Table KERNEL");

    let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();

    // if let Some(soc_node) = fdt.find_node("/soc") {
    //     let cell_sizes = soc_node.cell_sizes();
    //     assert_eq!(cell_sizes.size_cells, 2);
    //     assert_eq!(cell_sizes.address_cells, 2);
    //
    //     for child in soc_node.children() {
    //         if child.name.contains("serial") {
    //             // child.property()
    //         }
    //     }
    // }

    // Got these addresses from the parsing of the dtb from QEMU because it's quicker to test than parsing the dtb
    // println!("Setup UART0 Paging");
    //
    // const UART0: u64 = 0x10000000;
    // kernel_page_table.map_pages(
    //     VirtualAddr(UART0),
    //     PhysicalAddr(UART0),
    //     PAGE_SIZE, // Real Size 256
    //     PTE_READ | PTE_WRITE,
    //     0
    // );

    // Not needed
    // println!("Setup VIRTIO0 Paging");
    //
    // const VIRTIO0: usize = 0x10001000;
    // kernel_page_table.map_page_nosatp(
    //     VirtualAddr(VIRTIO0),
    //     PhysicalAddr(VIRTIO0),
    //     PAGE_SIZE,
    //     PTE_READ | PTE_WRITE,
    // );

    // println!("Setup PLIC Paging");
    //
    // const PLIC: u64 = 0x0c000000;
    // kernel_page_table.map_pages(
    //     VirtualAddr(PLIC),
    //     PhysicalAddr(PLIC),
    //     0x600000,
    //     PTE_READ | PTE_WRITE,
    //     0
    // );

    println!("Setup Kernel Code Paging");

    const KERNEL_BASE: u64 = 0x80200000; // From the linker
    let kernel_text_end_addr = page_round_up(unsafe { &_kernel_end_text as *const u8 as u64 });
    assert!(KERNEL_BASE < kernel_text_end_addr);
    println!(
        "Mapping kernel from 0x{:x} - 0x{:x}",
        KERNEL_BASE, kernel_text_end_addr
    );
    kernel_page_table.map_pages(
        VirtualAddr::new(KERNEL_BASE),
        PhysicalAddr::new(KERNEL_BASE),
        (kernel_text_end_addr - KERNEL_BASE) as usize,
        PTEPermission::read() | PTEPermission::execute(),
        0,
    );

    println!("Setup Kernel Data Paging");

    let kernel_end_addr = page_round_up(unsafe { &_kernel_end as *const u8 as u64 });
    assert!(kernel_end_addr > kernel_text_end_addr);
    kernel_page_table.map_pages(
        VirtualAddr::new(kernel_text_end_addr),
        PhysicalAddr::new(kernel_text_end_addr),
        (kernel_end_addr - kernel_text_end_addr) as usize,
        PTEPermission::read() | PTEPermission::write(),
        0,
    );

    println!("Setup Memory Paging");

    let start_memory = PAGE_ALLOCATOR.start_addr();
    let memory_size = PAGE_ALLOCATOR.end_addr() - start_memory;
    assert!(memory_size > 0);
    assert!(start_memory >= kernel_end_addr as usize);
    kernel_page_table.map_pages(
        VirtualAddr::new(start_memory as u64),
        PhysicalAddr::new(start_memory as u64),
        memory_size,
        PTEPermission::read() | PTEPermission::write(),
        2,
    );

    let trampoline_addr = page_round_up(unsafe { &_trampoline as *const u8 as u64 });

    println!("Setup Trampoline: 0x{:x}", trampoline_addr);

    let start_memory = PAGE_ALLOCATOR.start_addr();
    let memory_size = PAGE_ALLOCATOR.end_addr() - start_memory;
    assert!(memory_size > 0);
    assert!(start_memory >= kernel_end_addr as usize);
    kernel_page_table.map_pages(
        TRAMPOLINE,
        PhysicalAddr::new(trampoline_addr),
        PAGE_SIZE,
        PTEPermission::read() | PTEPermission::execute(),
        0,
    );

    println!("Setup Page Table finished");

    let kernel_page_table_addr = *kernel_page_table.deref() as *const PageTable as u64;

    unsafe {
        // Enable paging
        riscv::asm::sfence_vma_all();
        riscv::register::satp::set(
            Mode::Sv39,
            0,
            PhysicalAddr::new(kernel_page_table_addr).ppn().get() as usize,
        );
        riscv::asm::sfence_vma_all();
    }

    println!("Setup Kernel Paging Finished");
}

pub(crate) fn new_user_page_table(proc_trap_frame: &TrapFrame) -> Box<PageTable> {
    let mut page_table = Box::new(PageTable::new());
    // let mut page_table: &mut PageTable =
    //     unsafe { &mut *(PAGE_ALLOCATOR.kalloc().unwrap().cast().as_ptr()) };
    // TODO : Add trapframe and trampoline
    let trampoline_addr = page_round_up(unsafe { &_trampoline as *const u8 as u64 });

    page_table.map_pages(
        TRAMPOLINE,
        PhysicalAddr::new(trampoline_addr),
        PAGE_SIZE,
        PTEPermission::read() | PTEPermission::execute(),
        0,
    );

    page_table.map_pages(
        TRAPFRAME,
        PhysicalAddr::new(proc_trap_frame as *const _ as u64),
        PAGE_SIZE,
        PTEPermission::read() | PTEPermission::write(),
        0,
    );

    // NonNull::new(page_table).unwrap()
    page_table
}
