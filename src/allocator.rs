use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::println;
use core::alloc::{GlobalAlloc, Layout};
use core::ops::DerefMut;
use core::ptr::NonNull;
use core::usize;
use spin::lazy::Lazy;
use spin::Mutex;
use crate::vm::KERNEL_PAGE_TABLE;
use crate::vm::repr::{PhysicalAddr, PTE_READ, PTE_WRITE, VirtualAddr};

struct FreeMemoryNode {
    next: Option<NonNull<FreeMemoryNode>>,
    size: usize,
}

struct MyAllocator {
    start_address: PhysicalAddr,
    end_address: PhysicalAddr,
    allocated: usize,
    nodes: Option<NonNull<FreeMemoryNode>>,
}

struct MyGlobalAllocator(Lazy<Mutex<MyAllocator>>);

impl MyGlobalAllocator {
    pub fn init(&mut self, start_addr: PhysicalAddr, end_addr: PhysicalAddr) {
        let mut alloc = self.0.lock();
        alloc.start_address = start_addr;
        alloc.end_address = end_addr;
        let first_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let va = VirtualAddr(usize::from(first_page.addr()) as u64);
        let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();

        kernel_page_table.map_pages(
            va,
            alloc.start_address.clone(),
            PAGE_SIZE,
            PTE_READ | PTE_WRITE,
            0
        );

        alloc.allocated = PAGE_SIZE;

        let mut node: NonNull<FreeMemoryNode> = first_page.cast();
        unsafe {
            node.as_mut().size = PAGE_SIZE;
            node.as_mut().next = None;
        }
        alloc.nodes = Some(node);
    }
}

unsafe impl GlobalAlloc for MyGlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc = self.0.lock();
        let mut nodes = alloc.nodes;
        let mut last_node_opt: NonNull<FreeMemoryNode> = alloc.nodes.unwrap(); // Should panic if no regions
        while let Some(mut non_null_node) = nodes {
            let node = non_null_node;

            if node.as_ref().size >= layout.size() {
                if layout.size() + core::mem::size_of::<FreeMemoryNode>() > node.as_ref().size {
                    let mut new_node = NonNull::new(last_node_opt.as_ptr().byte_offset(layout.size() as isize)).unwrap();
                    *new_node.as_mut() = FreeMemoryNode { next: node.as_ref().next, size: node.as_ref().size - layout.size() };
                    last_node_opt.as_mut().next = Some(new_node);
                }
                else {
                    last_node_opt.as_mut().next = node.as_ref().next;
                }
                return usize::from(node.addr()) as *mut u8
            }
            last_node_opt = node;
            nodes = non_null_node.as_mut().next;
        }
        // TODO : Should merge contiguous nodes or allocate more pages
        panic!("Not enough space on the heap");
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}

#[global_allocator]
static mut ALLOCATOR: MyGlobalAllocator = MyGlobalAllocator(Lazy::new(|| {
    Mutex::new(MyAllocator {
        start_address: PhysicalAddr(0),
        end_address: PhysicalAddr(0),
        allocated: 0,
        nodes: None,
    })
}));

pub fn init_heap() {
    println!("Init heap");

    unsafe {
        ALLOCATOR.init(PhysicalAddr(0x100000000), PhysicalAddr(0x110000000));
    }
}
