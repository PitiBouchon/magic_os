use crate::{print, println};
use fdt::node::MemoryReservation;
use fdt::standard_nodes::MemoryRegion;
use fdt::Fdt;

extern "C" {
    static _kernel_end: u8;
}

#[derive(Debug, Copy, Clone)]
pub struct MyMemoryRegion {
    pub address: u64,
    pub size: u64,
}

impl TryFrom<MemoryRegion> for MyMemoryRegion {
    type Error = ();

    fn try_from(value: MemoryRegion) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.starting_address as u64,
            size: value.size.ok_or(())? as u64,
        })
    }
}

impl From<MemoryReservation> for MyMemoryRegion {
    fn from(value: MemoryReservation) -> Self {
        Self {
            address: value.address() as u64,
            size: value.size() as u64,
        }
    }
}

fn reserved_memory<'a>(fdt: &'a Fdt) -> impl IntoIterator<Item = MyMemoryRegion> + 'a {
    fdt
        .all_nodes()
        .find(|node| node.name == "reserved-memory")
        .map(|reserved_memory_node| {
            reserved_memory_node
                .children()
                .map(move |reserved_memory| {
                    let reg_prop = reserved_memory
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

                    let address =
                        unsafe { core::mem::transmute::<[u8; 8], u64>(*addr_values) }.to_be();
                    let size =
                        unsafe { core::mem::transmute::<[u8; 8], u64>(*size_values) }.to_be();

                    println!("Addr = {} | Size = {}", address, size);

                    MyMemoryRegion { address, size }
                })
        })
        .unwrap()
        .chain(fdt
            .memory_reservations()
            .map(|memory_reservation| memory_reservation.into())
        )
}

pub fn get_free_memory(fdt: &Fdt) -> MyMemoryRegion {
    let kernel_end_addr = unsafe { &_kernel_end as *const u8 as u64 };

    for memory_region in fdt.memory().regions() {
        if let Some(memory_size) = memory_region.size {
            let starting_address = (memory_region.starting_address as u64).max(kernel_end_addr);
            let end_address = ((memory_region.starting_address as usize) + memory_size) as u64;
            // Assert the reserved memory don't overlap with this memory (otherwise we juste panic for now)
            for reserved_region in reserved_memory(fdt) {
                assert!(reserved_region.address > end_address || reserved_region.address + reserved_region.size < starting_address);
            }
            // Return the first memory found but it should be more
            return MyMemoryRegion { address: starting_address, size:  memory_size as u64};
        }
    }

    panic!("No free memory found")
}
