use crate::physical_memory_manager::MyMemoryRegion;
use core::alloc::AllocError;
use core::ops::DerefMut;
use spin::Mutex;

pub const PAGE_SIZE: usize = 4096;

pub const fn page_round_down(addr: u64) -> u64 {
    addr & !(PAGE_SIZE as u64 - 1)
}

pub const fn page_round_up(addr: u64) -> u64 {
    page_round_down(addr + PAGE_SIZE as u64 - 1)
}

struct Node {
    next: Option<usize>,
}

pub unsafe fn init_page_allocator(free_memory_region: MyMemoryRegion) {
    PAGE_ALLOCATOR.0.lock().init(free_memory_region);
}

// Only used for the PAGE_ALLOCATOR static
struct PageAllocator {
    start: usize,
    end: usize,
    node: Option<usize>,
}

pub struct StaticPageAllocator(Mutex<PageAllocator>);

impl PageAllocator {
    fn init(&mut self, free_memory_region: MyMemoryRegion) {
        let start_memory_addr = page_round_up(free_memory_region.address);
        let end_memory_addr =
            page_round_down(free_memory_region.address + free_memory_region.size);
        self.start = start_memory_addr as usize;
        let mut old_node = start_memory_addr as *mut Node;
        unsafe {
            (*old_node).next = None;
        }
        // TODO : Find the error
        for current_addr in (start_memory_addr..)
            .step_by(PAGE_SIZE)
            .take_while(|&addr| addr + (PAGE_SIZE as u64) < end_memory_addr)
        {
            let mut next_node = current_addr as *mut Node;
            unsafe {
                (*next_node).next = None;
                (*old_node).next = Some(current_addr as usize);
            }
            old_node = next_node;
        }
        self.end = old_node as usize;
        self.node = Some(start_memory_addr as usize);
    }
}

pub static PAGE_ALLOCATOR: StaticPageAllocator = StaticPageAllocator(Mutex::new(PageAllocator {
    start: 0,
    end: 0,
    node: None,
}));

impl StaticPageAllocator {
    pub fn start_addr(&self) -> usize {
        self.0.lock().start
    }

    pub fn end_addr(&self) -> usize {
        self.0.lock().end
    }

    pub fn kalloc(&self) -> Result<usize, AllocError> {
        let mut alloc = self.0.lock();
        let first_node_addr = alloc.node.ok_or(AllocError)?;
        unsafe {
            alloc.node = (*(first_node_addr as *mut Node)).next;
        }

        unsafe {
            memset(first_node_addr, PAGE_SIZE, 0);
        }
        Ok(first_node_addr)
    }

    pub fn kfree(&self, physical_address: usize) {
        assert_eq!(physical_address % PAGE_SIZE, 0);
        let mut alloc = self.0.lock();
        assert!(physical_address >= alloc.start);
        assert!(physical_address < alloc.end);

        let new_node = physical_address as *mut Node;
        unsafe {
            (*new_node).next = alloc.node.map(|node| (node as *mut Node) as usize);
            alloc.node = (*new_node).next;
        }
    }
}

pub unsafe fn memset(addr: usize, size: usize, value: u8) {
    for addr in addr..(addr + size) {
        unsafe {
            let ptr = addr as *mut u8;
            ptr.write(value);
        }
    }
}
