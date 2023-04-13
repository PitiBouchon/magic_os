mod repr;
mod page_table;

use crate::kalloc::{page_round_down, page_round_up, PAGE_ALLOCATOR, PAGE_SIZE};
use crate::{kernelvec, println};
use page_table::PageTable;
use repr::{VirtualAddr, PhysicalAddr, Permission, PTE_READ, PTE_EXECUTE, PTE_WRITE};
use fdt::Fdt;
use riscv::register::satp::Mode;

extern "C" {
    static _kernel_end_text: u8;
    static _kernel_end: u8;
}

static mut KERNEL_PAGE: Option<&mut PageTable> = None;

pub fn init_paging(fdt: &Fdt) {
    // TODO : use Fdt for mapping the pages
    let kernel_page_table: &mut PageTable =
        unsafe { &mut *(PAGE_ALLOCATOR.kalloc().unwrap() as *mut PageTable) };

    println!("Setup Page Table KERNEL");

    // Got these addresses from the parsing of the dtb from QEMU because it's quicker to test than parsing the dtb
    println!("Setup UART0 Paging");

    const UART0: u64 = 0x10000000;
    kernel_page_table.map_pages(
        VirtualAddr(UART0),
        PhysicalAddr(UART0),
        PAGE_SIZE, // Real Size 256
        PTE_READ | PTE_WRITE,
        0
    );

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

    println!("Setup PLIC Paging");

    const PLIC: u64 = 0x0c000000;
    kernel_page_table.map_pages(
        VirtualAddr(PLIC),
        PhysicalAddr(PLIC),
        0x600000,
        PTE_READ | PTE_WRITE,
        0
    );

    println!("Setup Kernel Code Paging");

    const KERNEL_BASE: u64 = 0x80200000; // From the linker
    let kernel_text_end_addr = page_round_up(unsafe { &_kernel_end_text as *const u8 as u64 });
    assert!(KERNEL_BASE < kernel_text_end_addr);
    println!(
        "Mapping kernel from 0x{:x} - 0x{:x}",
        KERNEL_BASE, kernel_text_end_addr
    );
    kernel_page_table.map_pages(
        VirtualAddr(KERNEL_BASE),
        PhysicalAddr(KERNEL_BASE),
        (kernel_text_end_addr - KERNEL_BASE) as usize,
        PTE_READ | PTE_EXECUTE,
        0
    );

    println!("Setup Kernel Data Paging");

    let kernel_end_addr = page_round_up(unsafe { &_kernel_end as *const u8 as u64 });
    assert!(kernel_end_addr > kernel_text_end_addr);
    kernel_page_table.map_pages(
        VirtualAddr(kernel_text_end_addr),
        PhysicalAddr(kernel_text_end_addr),
        (kernel_end_addr - kernel_text_end_addr) as usize,
        PTE_READ | PTE_WRITE,
        0
    );

    println!("Setup Memory Paging");

    let start_memory = PAGE_ALLOCATOR.start_addr();
    let memory_size = PAGE_ALLOCATOR.end_addr() - start_memory;
    assert!(memory_size > 0);
    assert!(start_memory >= kernel_end_addr as usize);
    kernel_page_table.map_pages(
        VirtualAddr(start_memory as u64),
        PhysicalAddr(start_memory as u64),
        memory_size,
        PTE_READ | PTE_WRITE,
        0
    );

    println!("Setup Page Table finished");

    let kernel_page_table_addr = kernel_page_table as *const PageTable as u64;

    unsafe {
        KERNEL_PAGE = Some(kernel_page_table);

        // Enable paging
        riscv::asm::sfence_vma_all();
        riscv::register::satp::set(Mode::Sv39, 0, PhysicalAddr(kernel_page_table_addr).ppn().get() as usize);
        riscv::asm::sfence_vma_all();
    }

    println!("Setup Kernel Paging Finished");
}
