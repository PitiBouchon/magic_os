use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::println;
use crate::vm::repr::{PhysicalAddr, VirtualAddr, PTE_READ, PTE_WRITE};
use crate::vm::KERNEL_PAGE_TABLE;
use core::alloc::{GlobalAlloc, Layout};
use core::ops::DerefMut;
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
        let va = VirtualAddr(usize::from(first_page.addr()) as u64);
        let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();
        let (pa, _perm) = kernel_page_table.get_phys_addr_perm(&va);

        kernel_page_table.map_pages(
            alloc.start_address.clone(),
            pa,
            PAGE_SIZE,
            PTE_READ | PTE_WRITE,
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
        // TODO : respect alignment of layout
        assert_eq!(layout.align(), 1);
        let mut alloc = self.0.lock();
        let mut nodes = &mut alloc.nodes;
        while let Some(mut non_null_node) = *nodes {
                let node = non_null_node.as_ref();
                if node.size >= layout.size() {
                    if layout.size() + core::mem::size_of::<FreeMemoryNode>() <= node.size {
                        let ptr_new_node = non_null_node.as_ptr().byte_add(layout.size());
                        ptr_new_node.write_unaligned(FreeMemoryNode {
                            next: node.next,
                            size: node.size - layout.size(),
                        });
                        let new_node = NonNull::new(ptr_new_node).unwrap();
                        *nodes = Some(new_node);
                    } else {
                        *nodes = node.next;
                    }
                    return usize::from(non_null_node.addr()) as *mut u8;
                }
                nodes = &mut non_null_node.as_mut().next;
        }

        // TODO : Should merge contiguous nodes or allocate more pages
        panic!("Not enough space on the heap");
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // TODO : respect alignment of layout
        assert_eq!(layout.align(), 1);
        let mut alloc = self.0.lock();
        let mut nodes = &mut alloc.nodes;
        while let Some(mut non_null_node) = *nodes {
            let node = non_null_node.as_mut();
            if ptr.byte_add(layout.size()) == non_null_node.as_ptr().cast() {
                // The memory deallocated is at the beginning of the FreeMemoryNode
                let merged_size = node.size + layout.size();
                let ptr_node = ptr.cast::<FreeMemoryNode>();
                ptr_node.write_unaligned(FreeMemoryNode {
                    next: node.next,
                    size: merged_size
                });
                *nodes = Some(NonNull::new(ptr_node).unwrap());
                return
            }
            if non_null_node.as_ptr().byte_add(non_null_node.as_ref().size).cast() == ptr {
                // The memory deallocated is at the end of the FreeMemoryNode
                node.size += layout.size();
                return
            }
            nodes = &mut non_null_node.as_mut().next;
            if layout.size() >= core::mem::size_of::<FreeMemoryNode>() {
                let new_node_ptr = ptr.cast::<FreeMemoryNode>();
                new_node_ptr.write_unaligned(
                    FreeMemoryNode {
                        next: alloc.nodes,
                        size: layout.size(),
                    },
                );
                let new_node = NonNull::new(new_node_ptr).unwrap();
                alloc.nodes = Some(new_node);
                return
            }
            // TODO : Handle small region deallocated
            panic!("Deallocating too small region")
        }
    }
}

#[global_allocator]
static mut ALLOCATOR: MyGlobalAllocator = MyGlobalAllocator(Lazy::new(|| {
    Mutex::new(MyAllocator {
        start_address: VirtualAddr(0),
        end_address: VirtualAddr(0),
        allocated: 0,
        nodes: None,
    })
}));

pub fn init_heap() {
    println!("Init heap");

    unsafe {
        ALLOCATOR.init(VirtualAddr(0x100000000), VirtualAddr(0x110000000));
    }
}
