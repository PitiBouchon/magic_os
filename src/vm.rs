use fdt::Fdt;
use riscv::register::satp::Mode;
use crate::kalloc::{PAGE_ALLOCATOR, page_round_down, page_round_up, PAGE_SIZE};
use crate::println;

// 4096 bytes (PAGE_SIZE) / 8 bytes (64 bits) per entry = 512 entries
const ENTRY_COUNT: usize = 512;

const PTE_VALID: u8 = 0b0000_0001;
const PTE_READ: u8 = 0b0000_0010;
const PTE_WRITE: u8 = 0b0000_0100;
const PTE_EXECUTE: u8 = 0b0000_1000;
const PTE_USER: u8 = 0b0001_0000;
const PTE_GLOBAL: u8 = 0b0010_0000;
const PTE_ACCESS: u8 = 0b0100_0000;
const PTE_DIRTY: u8 = 0b1000_0000;

extern "C" {
    static _kernel_end: u8;
}

#[derive(Debug, Copy, Clone)]
struct PhysicalAddr(usize);

#[derive(Debug, Copy, Clone)]
struct VirtualAddr(usize);

impl VirtualAddr {
    fn virtual_page_numbers(&self) -> [usize; 3] {
        [
            (self.0 >> 12) & 0b111111111,
            (self.0 >> 12 >> 9) & 0b111111111,
            (self.0 >> 12 >> 9 >> 9) & 0b111111111,
        ]
    }

    fn page_round_down(self) -> Self {
        VirtualAddr(page_round_down(self.0))
    }

    fn page_round_up(self) -> Self {
        VirtualAddr(page_round_up(self.0))
    }
}

#[derive(Copy, Clone)]
struct PageTableEntry(usize);

impl PageTableEntry {
    fn permission(&self) -> u8 {
        (self.0 & 0b1111_1111) as u8
    }

    fn addr(&self) -> PhysicalAddr {
        PhysicalAddr((self.0 >> 10) << 12) // & 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
    }

    fn set_addr(&mut self, phys_addr: PhysicalAddr, perm: u8) {
        self.0 = ((phys_addr.0 >> 12) << 10) | perm as usize;
    }
}

struct PageTable([PageTableEntry; ENTRY_COUNT]);

impl PageTable {
    // Map a page when paging is still not enable
    // TODO : We could check if KERNEL_BASE == None to see if paging has been enabled maybe ?
    fn map_page_nosatp(&mut self, va: VirtualAddr, mut pa: PhysicalAddr, size: usize, perm: u8) {
        let mut va_current = va.page_round_down();
        let va_end = VirtualAddr(va_current.0 + size - 1).page_round_down();
        println!(
            "va_current: {:?} | va_end: {:?}",
            va_current.0,
            va_end.0 + size
        );
        loop {
            // println!("va_current: {}", va_current.0);
            let page_table_entry = self.walk_alloc(&va_current, perm);
            page_table_entry.set_addr(pa, perm | PTE_VALID);
            if va_current.0 == va_end.0 {
                break;
            }
            pa.0 += PAGE_SIZE;
            va_current.0 += PAGE_SIZE;
        }
    }

    fn walk_alloc(&mut self, va: &VirtualAddr, perm: u8) -> &mut PageTableEntry {
        let page_numbers = va.virtual_page_numbers(); // TODO : Maybe this should be reversed instead
        // println!("Page numbers: {:?}", page_numbers);
        let mut page_table = &mut self.0;
        let mut entry = &mut page_table[page_numbers[2]];
        for i in (0..2).rev() {
            // println!("Level {}, {}", i, entry.0);
            if entry.permission() & PTE_VALID == 0 {
                // This entry is not valid
                let addr = PAGE_ALLOCATOR.kalloc().unwrap();
                page_table = unsafe { &mut *(addr as *mut [PageTableEntry; ENTRY_COUNT]) };
                let page_table_addr = page_table.as_mut_ptr() as usize;
                entry.set_addr(PhysicalAddr(page_table_addr), perm | PTE_VALID);
                // println!("Setup new page: {} | {}", page_table_addr, entry.0);
            } else {
                // println!("Follow page table: {} | {}", entry.addr().0, entry.0);
                page_table = unsafe { &mut *(entry.addr().0 as *mut [PageTableEntry; ENTRY_COUNT]) };
            }
            entry = &mut page_table[page_numbers[i]];
        }
        entry
    }
}

static mut KERNEL_PAGE: Option<&mut PageTable> = None;

pub fn init_paging(_fdt: &Fdt) {
    let kernel_page_table: &mut PageTable = unsafe { &mut *(PAGE_ALLOCATOR.kalloc().unwrap() as *mut PageTable) };

    println!("Setup Page Table KERNEL");

    // println!("Setup UART0 Paging");
    //
    // Got these addresses from xv6-riscv
    // const UART0: usize = 0x10000000;
    // kernel_page_table.map_page_nosatp(
    //     VirtualAddr(UART0),
    //     PhysicalAddr(UART0),
    //     PAGE_SIZE,
    //     PTE_READ | PTE_WRITE,
    // );
    //
    // println!("Setup VIRTIO0 Paging");
    //
    // const VIRTIO0: usize = 0x10001000;
    // kernel_page_table.map_page_nosatp(
    //     VirtualAddr(VIRTIO0),
    //     PhysicalAddr(VIRTIO0),
    //     PAGE_SIZE,
    //     PTE_READ | PTE_WRITE,
    // );
    //
    // println!("Setup PLIC Paging");
    //
    // const PLIC: usize = 0x0c000000;
    // kernel_page_table.map_page_nosatp(
    //     VirtualAddr(PLIC),
    //     PhysicalAddr(PLIC),
    //     0x400000,
    //     PTE_READ | PTE_WRITE,
    // );

    println!("Setup Kernel Code Paging");

    const KERNEL_BASE: usize = 0x80200000; // From the linker
    let kernel_end_addr = unsafe { &_kernel_end as *const u8 as usize };
    assert!(KERNEL_BASE < kernel_end_addr);
    kernel_page_table.map_page_nosatp(
        VirtualAddr(KERNEL_BASE),
        PhysicalAddr(KERNEL_BASE),
        kernel_end_addr - KERNEL_BASE,
        PTE_READ | PTE_EXECUTE,
    );

    println!("Setup Memory Paging");

    let start_memory = PAGE_ALLOCATOR.start_addr();
    let memory_size = PAGE_ALLOCATOR.end_addr() - start_memory;
    assert!(memory_size > 0);
    assert!(start_memory > kernel_end_addr);
    kernel_page_table.map_page_nosatp(
        VirtualAddr(start_memory),
        PhysicalAddr(start_memory),
        memory_size,
        PTE_READ | PTE_WRITE,
    );

    println!("Setup Page Table finished");

    let kernel_page_table_addr = kernel_page_table as *const PageTable as usize;
    unsafe {
        KERNEL_PAGE = Some(kernel_page_table);

        // Enable paging
        riscv::asm::sfence_vma_all();
        riscv::register::satp::set(Mode::Sv39, 0, kernel_page_table_addr);
        riscv::asm::sfence_vma_all();
    }

    println!("Setup Kernel Paging Finished");

}
