use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::vm::page_table::addr::{PhysicalAddr, VirtualAddr, VirtualPageNumber};
use crate::vm::page_table::entry::perm::PTEPermission;
use crate::vm::page_table::entry::{EntryKind, PageTableEntry};

pub mod addr;
pub mod entry;

// 4096 bytes (PAGE_SIZE) / 8 bytes (64 bits) per entry = 512 entries
const ENTRY_COUNT: u16 = 512;

#[derive(Debug)]
#[repr(align(4096))]
pub(crate) struct PageTable([PageTableEntry; ENTRY_COUNT as usize]);

impl PageTable {
    pub const fn new() -> Self {
        const ZERO_ENTRY: PageTableEntry = PageTableEntry(0);
        Self([ZERO_ENTRY; ENTRY_COUNT as usize])
    }

    fn get_entry_mut(&mut self, vpn: VirtualPageNumber) -> &mut PageTableEntry {
        &mut self.0[vpn.0 as usize]
    }

    fn get_entry(&self, vpn: VirtualPageNumber) -> &PageTableEntry {
        &self.0[vpn.0 as usize]
    }

    pub fn get_phys_addr_perm(&self, va: &VirtualAddr) -> (PhysicalAddr, PTEPermission) {
        let page_numbers = va.virtual_page_numbers().into_iter().rev();
        let mut page_table = self;

        for vpn in page_numbers {
            let entry = page_table.get_entry(vpn);
            match entry.kind() {
                EntryKind::Leaf => {
                    return (
                        entry.convert_to_physical_addr(&va.page_offset()),
                        entry.perm(),
                    );
                }
                EntryKind::Branch(page_table_addr) => {
                    let new_page_table = unsafe { &*(page_table_addr.0 as *const PageTable) };
                    page_table = new_page_table;
                }
                EntryKind::NotValid => panic!("IMPOSSIBLE"),
            }
        }

        panic!("IMPOSSIBLE")
    }

    pub fn map_pages(
        &mut self,
        mut va: VirtualAddr,
        mut pa: PhysicalAddr,
        size: usize,
        perm: PTEPermission,
        _rsw: u8,
    ) {
        assert!(size > 0);
        let va_end = va.add_offset(size as u64).page_round_up();

        while va != va_end {
            let page_table_entry_leaf = self.walk_alloc(&va);
            // assert!(!page_table_entry_leaf.is_valid());
            // assert!(page_table_entry_leaf.is_zero());
            *page_table_entry_leaf =
                PageTableEntry::new(pa.ppn(), 0, PTEPermission::valid() | perm);
            pa.0 += PAGE_SIZE as u64;
            va = va.add_offset(PAGE_SIZE as u64);
        }
    }

    // // TODO : Unmap only one page for now (should do more)
    // pub fn unmap_pages(&mut self, mut va: VirtualAddr) {
    //     let page_table_entry_leaf = self.walk_alloc(&va);
    //     assert!(page_table_entry_leaf.is_valid());
    //     *page_table_entry_leaf = PageTableEntry::new_zero();
    // }

    pub fn walk_alloc(&mut self, va: &VirtualAddr) -> &mut PageTableEntry {
        let mut page_numbers = va.virtual_page_numbers().into_iter().rev();
        let mut page_table = self;
        let mut entry = page_table.get_entry_mut(page_numbers.next().unwrap());

        for vpn in page_numbers {
            match entry.kind() {
                EntryKind::Leaf => break,
                EntryKind::Branch(page_table_addr) => {
                    let new_page_table = unsafe { &mut *(page_table_addr.0 as *mut PageTable) };
                    page_table = new_page_table;
                }
                EntryKind::NotValid => {
                    // Allocate a page for a new PageTable
                    let new_page_table_addr = PAGE_ALLOCATOR.kalloc().unwrap().cast().as_ptr();
                    let new_page_table = unsafe { &mut *(new_page_table_addr as *mut PageTable) };
                    *entry = PageTableEntry::new(
                        PhysicalAddr(new_page_table_addr as u64).ppn(),
                        0,
                        PTEPermission::valid(),
                    );
                    page_table = new_page_table;
                }
            }
            entry = page_table.get_entry_mut(vpn);
        }

        entry
    }
}
