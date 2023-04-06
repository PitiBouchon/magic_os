use crate::println;
use fdt::Fdt;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

extern "C" {
    static _kernel_end: u8;
}

pub fn init_heap(fdt: &Fdt) -> Result<(usize, usize), ()> {
    let kernel_end_addr = unsafe { &_kernel_end as *const u8 as usize };
    assert_ne!(kernel_end_addr, 0);

    println!(
        "Memory Reservation Region: {}",
        fdt.memory_reservations().count()
    );

    for memory_region in fdt.memory().regions() {
        if let Some(size) = memory_region.size {
            assert_ne!(size, 0);
            let addr = unsafe { (memory_region.starting_address as usize).max(kernel_end_addr) };
            println!("Using heap from 0x{:x} to 0x{:x}", addr, addr + size);
            unsafe {
                ALLOCATOR.lock().init(addr as *mut u8, size);
            }
            return Ok((addr, size));
        }
    }

    Err(())
}
