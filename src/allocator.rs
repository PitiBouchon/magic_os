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

    unsafe fn print(mut nodes: &mut Option<NonNull<FreeMemoryNode>>, s: &str) {
        println!("{s}");
        while let Some(node) = nodes {
            println!("Node({:p} | 0x{:x})", node.as_ptr(), node.as_ref().size);
            nodes = &mut node.as_mut().next;
            println!("--");
        }
    }
}

pub const fn align_round_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

pub const fn align_round_up(addr: usize, align: usize) -> usize {
    align_round_down(addr + align - 1, align)
}

unsafe impl GlobalAlloc for MyGlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TODO : respect alignment
        assert_eq!(layout.align(), 1);
        let mut alloc = self.0.lock();
        let mut nodes = alloc.nodes;
        Self::print(&mut nodes, "[alloc] start");
        let mut last_node_opt: NonNull<FreeMemoryNode> = alloc.nodes.unwrap(); // Should panic if no regions
        while let Some(mut non_null_node) = nodes {
            let node = non_null_node;

            if node.as_ref().size >= layout.size() {
                if layout.size() + core::mem::size_of::<FreeMemoryNode>() < node.as_ref().size {
                    let ptr_new_node = last_node_opt.as_ptr().byte_add(layout.size());
                    // println!("A1: {:p}", ptr_new_node);
                    // println!("A2: {}", ptr_new_node.align_offset(core::mem::align_of::<FreeMemoryNode>()));
                    // ptr_new_node = ptr_new_node.add(ptr_new_node.align_offset(core::mem::align_of::<FreeMemoryNode>()));
                    // println!("B1: {:p}", ptr_new_node);
                    // println!("B2: {}", core::usize::MAX);
                    // println!("B3: {}", core::mem::align_of::<FreeMemoryNode>());
                    // assert!(ptr_new_node.is_aligned());
                    // ptr_new_node = align_round_up(ptr_new_node as usize, core::mem::align_of::<FreeMemoryNode>()) as *mut FreeMemoryNode;
                    core::ptr::write_unaligned(
                        ptr_new_node,
                        FreeMemoryNode {
                            next: node.as_ref().next,
                            size: node.as_ref().size - layout.size(),
                        },
                    );
                    let new_node = NonNull::new(ptr_new_node).unwrap();
                    if !alloc.nodes.contains(&last_node_opt) {
                        last_node_opt.as_mut().next = Some(new_node);
                    } else {
                        alloc.nodes = Some(new_node);
                    }
                } else if !alloc.nodes.contains(&last_node_opt) {
                    last_node_opt.as_mut().next = node.as_ref().next;
                } else {
                    alloc.nodes = node.as_ref().next;
                }
                Self::print(&mut alloc.nodes, "[alloc] end");
                return usize::from(node.addr()) as *mut u8;
            }
            last_node_opt = node;
            nodes = non_null_node.as_mut().next;
        }
        // TODO : Should merge contiguous nodes or allocate more pages
        panic!("Not enough space on the heap");
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Todo respect alignment
        assert_eq!(layout.align(), 1);
        println!("ptr: {:p}", ptr);
        println!("Size: {}", layout.size());
        println!("SizeOf<FreeMemoryNode>: {}", core::mem::size_of::<FreeMemoryNode>());
        let mut alloc = self.0.lock();
        let mut nodes = alloc.nodes;
        Self::print(&mut nodes, "[alloc] start");
        let mut last_node_opt: NonNull<FreeMemoryNode> = alloc.nodes.unwrap();
        while let Some(mut non_null_node) = nodes {
            if ptr.byte_add(layout.size()) == non_null_node.as_ptr().cast() {
                let merged_size = non_null_node.as_mut().size + layout.size();
                ptr.cast::<FreeMemoryNode>().write(FreeMemoryNode { next: non_null_node.as_ref().next, size: merged_size });
                last_node_opt.as_mut().next = Some(NonNull::new(ptr.cast()).unwrap());
                println!("Merged A");
                Self::print(&mut alloc.nodes, "[alloc] start");
                return
            }
            if non_null_node.as_ptr().byte_add(non_null_node.as_ref().size).cast() == ptr {
                non_null_node.as_mut().size += layout.size();
                println!("Merged B");
                return
            }

            nodes = non_null_node.as_mut().next;
        }
        if layout.size() >= core::mem::size_of::<FreeMemoryNode>() {
            core::ptr::write(
                ptr.cast(),
                FreeMemoryNode {
                    next: alloc.nodes,
                    size: layout.size(),
                },
            );
            let new_node = NonNull::new(ptr.cast()).unwrap();
            alloc.nodes = Some(new_node);
            return
        }
        panic!("")
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
