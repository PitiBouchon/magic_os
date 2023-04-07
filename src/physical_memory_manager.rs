use crate::{print, println};
use fdt::standard_nodes::MemoryRegion;
use fdt::Fdt;
use fdt::node::MemoryReservation;

pub fn init_memory(fdt: &Fdt) {
    let reserved_memory = fdt
        .all_nodes()
        .find(|node| node.name == "reserved-memory")
        .map(|reserved_memory_node| {
            reserved_memory_node
                .children()
                // Could use .contains instead
                // .find(|child_node| {
                //     child_node.name.len() > "mmode_resv".len()
                //         && &child_node.name[.."mmode_resv".len()] == "mmode_resv"
                // })
                .map(|mmode_rescv| {
                    let reg_prop = mmode_rescv
                        .properties()
                        .find(|prop| prop.name == "reg")
                        .unwrap();

                    let cell_size = reserved_memory_node.cell_sizes();
                    let address_cell = cell_size.address_cells;
                    let size_cell = cell_size.size_cells;
                    assert_eq!(
                        reg_prop.value.len(),
                        address_cell * size_cell * 4,
                        "Length of reg property invalid"
                    ); // Size of u32 = 4 * u8
                    assert_eq!(size_cell, 2, "size_cell should equal 2");
                    assert_eq!(address_cell, 2, "address_cell should equal 2");

                    let ([addr_values, size_values], reminder) =
                        reg_prop.value.as_chunks::<8>() else { panic!("Impossible") };

                    assert!(reminder.is_empty());

                    let addr =
                        unsafe { core::mem::transmute::<[u8; 8], u64>(*addr_values) }.to_be();
                    let size =
                        unsafe { core::mem::transmute::<[u8; 8], u64>(*size_values) }.to_be();

                    println!("Addr = {} | Size = {}", addr, size);

                    let a = MemoryRegion { starting_address: (), size: None };
                    let b: MemoryReservation = a.into();

                    // MemoryReservation {
                    //     address: (),
                    //     size: (),
                    // }
                })
        }).unwrap();

    // if let Some(reserved_memory) = open_sbi_memory_region_reserved {
    //     println!("Addr: {:x} | Size = {:?}", reserved_memory.starting_address as usize, reserved_memory.size);
    // }

    for reserve_node in fdt.memory_reservations().chain(memory_region_reserved) {
        println!("A");
    }
}
