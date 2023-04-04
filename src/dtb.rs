use crate::println;

pub unsafe fn init_dtb(dtb: usize) {
    println!("DTB at : 0x{dtb}");

    let fdt = fdt::Fdt::from_ptr(dtb as *const u8).unwrap();
    println!("CPUs : {}", fdt.cpus().count());

    for memory_region in fdt.memory().regions() {
        let start_addr = memory_region.starting_address as usize;
        let size = memory_region.size.unwrap();

        println!("Memory Region 0x{start_addr:x} end at 0x{:x}", start_addr + size);
    }

    for memory_reservation in fdt.memory_reservations() {
        let start_addr = memory_reservation.address() as usize;
        let size = memory_reservation.size();

        println!("Memory Reservation 0x{start_addr:x} end at 0x{:x}", start_addr + size);
    }
}
