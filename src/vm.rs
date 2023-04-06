// vm stands for Virtual Memory
// We use the Sv39 design (which limit the memory to 512Gb)
// VPN : Virtual Page Number
// PPN : Physical Page Number

use crate::println;
use alloc::boxed::Box;
use core::ops::DerefMut;
use riscv::register::satp::Mode;

extern "C" {
    static _kernel_end: u8;
}

pub const PAGE_SIZE: usize = 4096;

// 4096 bytes / 8 bytes per entry = 512 entries
const ENTRY_COUNT: usize = 512;

const PTE_VALID: u8 = 0b0000_0001;
const PTE_READ: u8 = 0b0000_0010;
const PTE_WRITE: u8 = 0b0000_0100;
const PTE_EXECUTE: u8 = 0b0000_1000;
const PTE_USER: u8 = 0b0001_0000;
const PTE_GLOBAL: u8 = 0b0010_0000;
const PTE_ACCESS: u8 = 0b0100_0000;
const PTE_DIRTY: u8 = 0b1000_0000;

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
        VirtualAddr(self.0 & !(PAGE_SIZE - 1))
    }
}

#[derive(Debug, Copy, Clone)]
struct PhysicalAddr(usize);

struct PageTableEntry(usize);

impl PageTableEntry {
    fn permission(&self) -> u8 {
        (self.0 & 0b1111_1111) as u8
    }

    fn addr(&self) -> usize {
        ((self.0 >> 10) << 12) & 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
    }

    fn set_addr(&mut self, phys_addr: PhysicalAddr, perm: u8) {
        self.0 = ((phys_addr.0 >> 12) << 10) | perm as usize;
    }
}

struct PageTable(Box<[PageTableEntry; ENTRY_COUNT]>);

const EMPTY_PAGE: PageTableEntry = PageTableEntry(0);

impl PageTable {
    fn new_empty() -> Self {
        Self(Box::new([EMPTY_PAGE; ENTRY_COUNT]))
    }

    fn map_page_nosatp(&mut self, va: VirtualAddr, mut pa: PhysicalAddr, size: usize, perm: u8) {
        let mut va_current = va.page_round_down();
        let va_end = VirtualAddr(va_current.0 + size - 1).page_round_down();
        println!("va_current: {:?} | va_end: {:?}", va_current.0, va_current.0 + size);
        loop {
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
        let page_numbers = va.virtual_page_numbers();
        let mut page_table = self.0.deref_mut();
        let mut entry = &mut page_table[page_numbers[0]];
        for i in 0..2 {
            println!("Level {}", i);
            if entry.permission() & PTE_VALID == 0 {
                println!("Setup new page");
                // This entry is not valid
                page_table = Box::leak(PageTable::new_empty().0);
                let page_table_addr = page_table.as_mut_ptr() as usize;
                entry.set_addr(PhysicalAddr(page_table_addr), perm | PTE_VALID);
            } else {
                page_table = unsafe { &mut *(entry.addr() as *mut PageTable) }.0.deref_mut();
            }
            println!("Follow page table");
            entry = &mut page_table[page_numbers[i]];
        }
        entry
    }
}

static mut KERNEL_PAGE: Option<PageTable> = None;

pub fn init_paging(start_heap: usize, heap_size: usize) {
    let mut kernel_page_table = PageTable::new_empty();

    println!("Setup Page Table KERNEL");

    const KERNEL_BASE: usize = 0x80200000; // From the linker
    let kernel_end_addr = unsafe { &_kernel_end as *const u8 as usize };
    assert!(KERNEL_BASE < kernel_end_addr);
    kernel_page_table.map_page_nosatp(
        VirtualAddr(KERNEL_BASE),
        PhysicalAddr(KERNEL_BASE),
        kernel_end_addr - KERNEL_BASE,
        PTE_READ | PTE_EXECUTE,
    );

    println!("Setup Page Table HEAP");

    kernel_page_table.map_page_nosatp(
        VirtualAddr(start_heap),
        PhysicalAddr(start_heap),
        heap_size,
        PTE_READ | PTE_WRITE,
    );

    println!("Setup Page Table finished");

    unsafe {
        let kernel_page_table_addr = &kernel_page_table as *const PageTable as usize;
        KERNEL_PAGE = Some(kernel_page_table);

        // Enable paging
        riscv::asm::sfence_vma_all();
        riscv::register::satp::set(Mode::Sv39, 0, kernel_page_table_addr);
        riscv::asm::sfence_vma_all();
    }

    println!("Setup Page Table variable finished");
}
