#![feature(pointer_byte_offsets)]
#![feature(strict_provenance)]
#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::NonNull;
use core::usize;
use page_alloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use page_table::entry::addr::VirtualAddr;
use page_table::entry::perm::PTEPermission;
use sbi_print::println;
use spin::lazy::Lazy;
use spin::{Mutex, MutexGuard};
use page_table::PageTable;

struct FreeMemoryNode {
    next: Option<NonNull<FreeMemoryNode>>,
    size: usize,
}

struct MyAllocator {
    start_address: VirtualAddr,
    end_address: VirtualAddr,
    allocated: usize,
    nodes: Option<NonNull<FreeMemoryNode>>,
    kernel_page_table: Option<&'static Mutex<&'static mut PageTable>>
}

struct MyGlobalAllocator(Lazy<Mutex<MyAllocator>>);

impl MyGlobalAllocator {
    pub fn init(&mut self, start_addr: VirtualAddr, end_addr: VirtualAddr, kernel_page_table: &'static Mutex<&'static mut PageTable>) {
        let mut alloc = self.0.lock();
        alloc.kernel_page_table = Some(kernel_page_table);
        alloc.start_address = start_addr;
        alloc.end_address = end_addr;
        let first_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let va = VirtualAddr::new(usize::from(first_page.addr()) as u64);
        let mut kernel_page_table = alloc.kernel_page_table.unwrap().lock();
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
        assert!(layout.align() > 0, "Align is 0");
        assert_eq!(
            layout.align() & (layout.align() - 1),
            0,
            "Align is not a power of 2"
        );
        let alloc_size = if layout.size() % size_of::<FreeMemoryNode>() != 0 {
            layout.size()
                + (size_of::<FreeMemoryNode>() - layout.size() % size_of::<FreeMemoryNode>())
        } else {
            layout.size()
        };
        assert_eq!(
            alloc_size % size_of::<FreeMemoryNode>(),
            0,
            "Alloc_size is not a multiple of size_of::<FreeMemoryNode>()"
        );
        let mut alloc = self.0.lock();
        let mut nodes = &mut alloc.nodes;
        while let Some(mut non_null_node) = *nodes {
            let mut ptr = usize::from(non_null_node.addr()) as *mut u8;
            let diff_align = ptr.align_offset(layout.align());
            ptr = ptr.byte_add(diff_align);
            let node = non_null_node.as_ref();
            if node.size >= alloc_size + diff_align {
                if alloc_size + size_of::<FreeMemoryNode>() <= node.size {
                    let ptr_new_node = non_null_node.as_ptr().byte_add(alloc_size + diff_align);
                    ptr_new_node.write_unaligned(FreeMemoryNode {
                        next: node.next,
                        size: node.size - layout.size() - diff_align,
                    });
                    let new_node = NonNull::new(ptr_new_node).unwrap();
                    *nodes = Some(new_node);
                } else {
                    *nodes = node.next;
                    assert_eq!(
                        alloc_size, node.size,
                        "Alloc_size should be equal to node.size here"
                    );
                }
                return ptr;
            }
            nodes = &mut non_null_node.as_mut().next;
        }

        // TODO : Should try to merge contiguous nodes ?
        println!("Not enough space on the heap, allocating one more page");

        let new_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let va = VirtualAddr::new(usize::from(new_page.addr()) as u64);

        let mut kernel_page_table = alloc.kernel_page_table.unwrap().lock();
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
        assert!(layout.align() > 0, "Align is 0");
        assert_eq!(
            layout.align() & (layout.align() - 1),
            0,
            "Align is not a power of 2"
        );
        let alloc_size = if layout.size() % size_of::<FreeMemoryNode>() != 0 {
            layout.size()
                + (size_of::<FreeMemoryNode>() - layout.size() % size_of::<FreeMemoryNode>())
        } else {
            layout.size()
        };
        assert_eq!(
            alloc_size % size_of::<FreeMemoryNode>(),
            0,
            "Alloc_size is not a multiple of size_of::<FreeMemoryNode>()"
        );
        let ptr_start = ptr.addr();
        let ptr_end = ptr.byte_add(alloc_size).addr();
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
        if alloc_size >= size_of::<FreeMemoryNode>() {
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
        kernel_page_table: None,
    })
}));

pub fn init_heap(kernel_page_table: &'static Mutex<&'static mut PageTable>) {
    println!("Init heap");

    unsafe {
        ALLOCATOR.init(VirtualAddr::new(0x100000000), VirtualAddr::new(0x110000000), kernel_page_table);
    }

    println!("End init heap !");
}
