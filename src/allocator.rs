use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::vm::page_table::addr::VirtualAddr;
use crate::vm::page_table::entry::perm::PTEPermission;
use crate::vm::KERNEL_PAGE_TABLE;
use crate::println;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::NonNull;
use core::usize;
use spin::lazy::Lazy;
use spin::Mutex;

struct FreeMemoryNode {
    next: Option<NonNull<FreeMemoryNode>>,
    size: usize,
}

struct MyAllocator {
    start_address: VirtualAddr,
    end_address: VirtualAddr,
    allocated: usize,
    nodes: Option<NonNull<FreeMemoryNode>>,
}

struct MyGlobalAllocator(Lazy<Mutex<MyAllocator>>);

impl MyGlobalAllocator {
    pub fn init(&mut self, start_addr: VirtualAddr, end_addr: VirtualAddr) {
        let mut alloc = self.0.lock();
        alloc.start_address = start_addr;
        alloc.end_address = end_addr;
        let first_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let va = VirtualAddr::new(usize::from(first_page.addr()) as u64);
        let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();
        let (pa, _perm) = kernel_page_table.get_phys_addr_perm(&va);

        kernel_page_table.map_pages(
            alloc.start_address.clone(),
            pa,
            PAGE_SIZE,
            PTEPermission::read() | PTEPermission::write(),
            0,
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
        let mut alloc = self.0.lock();
        let mut nodes = &mut alloc.nodes;
        while let Some(mut non_null_node) = *nodes {
            let mut ptr = usize::from(non_null_node.addr()) as *mut u8;
            let diff_align = ptr.align_offset(layout.align());
            ptr = ptr.byte_add(diff_align);
            let node = non_null_node.as_ref();
            if node.size >= layout.size() + diff_align {
                if layout.size() + core::mem::size_of::<FreeMemoryNode>() <= node.size {
                    let ptr_new_node = non_null_node.as_ptr().byte_add(layout.size() + diff_align);
                    ptr_new_node.write_unaligned(FreeMemoryNode {
                        next: node.next,
                        size: node.size - layout.size() - diff_align,
                    });
                    let new_node = NonNull::new(ptr_new_node).unwrap();
                    *nodes = Some(new_node);
                } else {
                    *nodes = node.next;
                }
                return ptr;
            }
            nodes = &mut non_null_node.as_mut().next;
        }

        // TODO : Should try to merge contiguous nodes ?
        println!("Not enough space on the heap, allocating one more page");

        let new_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let va = VirtualAddr::new(usize::from(new_page.addr()) as u64);

        let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();
        let (pa, _perm) = kernel_page_table.get_phys_addr_perm(&va);

        kernel_page_table.map_pages(
            alloc.start_address.add_offset(alloc.allocated as u64),
            pa,
            PAGE_SIZE,
            PTEPermission::read() | PTEPermission::write(),
            0,
        );

        alloc.allocated += PAGE_SIZE;

        let mut node: NonNull<FreeMemoryNode> = new_page.cast();
        unsafe {
            node.as_mut().size = PAGE_SIZE;
            node.as_mut().next = alloc.nodes;
        }
        alloc.nodes = Some(node);

        // TODO: know why do I need to drop alloc to release the lock ??
        drop(alloc);

        self.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // TODO : Is alignement useful here ?
        let ptr_start = ptr.addr();
        let ptr_end = ptr.byte_add(layout.size()).addr();
        let mut alloc = self.0.lock();
        let mut nodes = &mut alloc.nodes;
        while let Some(mut non_null_node) = *nodes {
            let node = non_null_node.as_mut();
            let start_addr = usize::from(non_null_node.addr());
            let end_addr = non_null_node.as_ptr().byte_add(node.size).addr();
            if (ptr_end..=(ptr_end + size_of::<FreeMemoryNode>())).contains(&start_addr) {
                // The memory deallocated is at the beginning of the FreeMemoryNode
                let ptr_node = ptr.cast::<FreeMemoryNode>();
                ptr_node.write_unaligned(FreeMemoryNode {
                    next: node.next,
                    size: end_addr - ptr_start,
                });
                *nodes = Some(NonNull::new(ptr_node).unwrap());
                return;
            }
            if (end_addr..=(end_addr + size_of::<FreeMemoryNode>())).contains(&ptr_start) {
                // The memory deallocated is at the end of the FreeMemoryNode
                node.size = ptr_end - start_addr;
                return;
            }
            nodes = &mut non_null_node.as_mut().next;
        }
        if layout.size() >= size_of::<FreeMemoryNode>() {
            println!("Merge NOWHERE");
            let new_node_ptr = ptr.cast::<FreeMemoryNode>();
            new_node_ptr.write_unaligned(FreeMemoryNode {
                next: alloc.nodes,
                size: layout.size(),
            });
            let new_node = NonNull::new(new_node_ptr).unwrap();
            alloc.nodes = Some(new_node);
            return;
        }
        panic!(
            "Deallocating too small region: {} bytes < {} bytes",
            layout.size(),
            size_of::<FreeMemoryNode>()
        )
    }
}

#[global_allocator]
static mut ALLOCATOR: MyGlobalAllocator = MyGlobalAllocator(Lazy::new(|| {
    Mutex::new(MyAllocator {
        start_address: VirtualAddr::new(0),
        end_address: VirtualAddr::new(0),
        allocated: 0,
        nodes: None,
    })
}));

pub fn init_heap() {
    println!("Init heap");

    unsafe {
        ALLOCATOR.init(VirtualAddr::new(0x100000000), VirtualAddr::new(0x110000000));
    }
}
