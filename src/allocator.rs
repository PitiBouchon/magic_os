use core::alloc::{GlobalAlloc, Layout};
use core::ops::DerefMut;
use core::ptr::NonNull;
use spin::Mutex;
use crate::kalloc::{PAGE_ALLOCATOR, PAGE_SIZE};
use crate::println;
use spin::lazy::Lazy;
use crate::vm::KERNEL_PAGE;

struct FreeMemoryNode {
    next: Option<core::ptr::NonNull<FreeMemoryNode>>,
    size: usize,
}

struct MyAllocator {
    nodes: Lazy<Mutex<Option<core::ptr::NonNull<FreeMemoryNode>>>>
}

impl MyAllocator {
    pub fn init(&mut self) {
        let first_page = PAGE_ALLOCATOR.kalloc().unwrap();
        let mut node = core::ptr::NonNull::new(first_page as *mut FreeMemoryNode).unwrap();
        unsafe {
            node.as_mut().size = PAGE_SIZE;
            node.as_mut().next = None;
        }
        *self.nodes.lock().deref_mut() = Some(node);
    }
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.nodes.lock();
        let mut nodes = alloc.deref_mut();
        let mut last_node_opt: Option<&mut FreeMemoryNode> = None;
        while let Some(mut non_null_node) = nodes {
            let node = non_null_node.as_mut();

            if node.size >= layout.size() {
                // if let Some(last_node) = last_node_opt {
                //     let last_ptr = last_node.next.unwrap().as_ptr();
                // } else {
                // }
                // return core::ptr::null_mut::<u8>()
            }
            last_node_opt = Some(node);
            nodes = &mut non_null_node.as_mut().next;
        }
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}

#[global_allocator]
static mut ALLOCATOR: MyAllocator = MyAllocator {
    nodes: Lazy::new(|| {Mutex::new(None) })
};

pub fn init_heap() {
    println!("Init heap");

    unsafe {
        ALLOCATOR.init();
    }
}
