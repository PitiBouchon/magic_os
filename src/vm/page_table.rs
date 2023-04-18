use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::println;
use crate::vm::repr::{EntryKind, PageOffset, PageTableEntry, Permission, PhysicalAddr, PTE_VALID, VirtualAddr, VirtualPageNumber};

// 4096 bytes (PAGE_SIZE) / 8 bytes (64 bits) per entry = 512 entries
const ENTRY_COUNT: u16 = 512;

#[derive(Debug)]
pub struct PageTable([PageTableEntry; ENTRY_COUNT as usize]);

impl PageTable {
    fn get_entry_mut(&mut self, vpn: VirtualPageNumber) -> &mut PageTableEntry {
        &mut self.0[vpn.0 as usize]
    }

    fn get_entry(&self, vpn: VirtualPageNumber) -> &PageTableEntry {
        &self.0[vpn.0 as usize]
    }

    pub fn get_phys_addr_perm(&self, va: &VirtualAddr) -> (PhysicalAddr, Permission) {
        let page_numbers = va.virtual_page_numbers().into_iter().rev();
        let mut page_table = self;

        for vpn in page_numbers {
            let entry = page_table.get_entry(vpn);
            match entry.kind() {
                EntryKind::Leaf => {
                    return ((entry, va.page_offset()).into(), entry.perm());
                }
                EntryKind::Branch(page_table_addr) => {
                    let new_page_table = unsafe { & *(page_table_addr.0 as *const PageTable) };
                    page_table = new_page_table;
                }
                EntryKind::NotValid => panic!("IMPOSSIBLE"),
            }
        }

        panic!("IMPOSSIBLE")
    }

    pub fn map_pages(&mut self, mut va: VirtualAddr, mut pa: PhysicalAddr, size: usize, perm: u8, rsw: u8) {
        assert!(size > 0);
        let va_end = VirtualAddr(va.0 + size as u64).page_round_up();

        while va != va_end {
            let page_table_entry_leaf = self.walk_alloc(&va);
            // assert!(!page_table_entry_leaf.is_valid());
            // assert!(page_table_entry_leaf.is_zero());
            *page_table_entry_leaf = PageTableEntry::new(pa.ppn(), 0, Permission(perm | PTE_VALID));
            pa.0 += PAGE_SIZE as u64;
            va.0 += PAGE_SIZE as u64;
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
                    *entry = PageTableEntry::new(PhysicalAddr(new_page_table_addr as u64).ppn(), 0, Permission(PTE_VALID));
                    page_table = new_page_table;
                }
            }
            entry = page_table.get_entry_mut(vpn);
        }

        entry
    }
}
