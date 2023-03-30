use core::ptr::slice_from_raw_parts;
use crate::sbi_print::{sbi_print_number_base10, sbi_print_str, sbi_println_number_base10, sbi_println_str};

// // Memory Reservation Block
// #[repr(C)]
// struct FdtReserveEntry {
//     address: u64,
//     size: u64,
// }
//
// // Structure Block
// const FDT_BEGIN_NODE: u32 = 0x00000001;
// const FDT_END_NODE: u32 = 0x00000002;
// const FDT_PROP: u32 = 0x00000003;
// const FDT_NOP: u32 = 0x00000004;
// const FDT_END: u32 = 0x00000009;

// See : https://devicetree-specification.readthedocs.io/en/v0.2/flattened-format.html
#[allow(dead_code)]
#[repr(C)]
pub struct FdtHeader {
    magic: u32,
    pub total_size: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

impl FdtHeader {
    pub unsafe fn init_fdt_header(dtb: usize) {
        sbi_println_str("Get FdtHeader");
        if dtb % 8 != 0 {
            panic!("FDT is not 8-byte aligned");
        }
        let fdt_header_le: &FdtHeader = &*(dtb as *const FdtHeader); // Unsafe

        let fdt_header = &FdtHeader {
            magic: fdt_header_le.magic.to_be(),
            total_size: fdt_header_le.total_size.to_be(),
            off_dt_struct: fdt_header_le.off_dt_struct.to_be(),
            off_dt_strings: fdt_header_le.off_dt_strings.to_be(),
            off_mem_rsvmap: fdt_header_le.off_mem_rsvmap.to_be(),
            version: fdt_header_le.version.to_be(),
            last_comp_version: fdt_header_le.last_comp_version.to_be(),
            boot_cpuid_phys: fdt_header_le.boot_cpuid_phys.to_be(),
            size_dt_strings: fdt_header_le.size_dt_strings.to_be(),
            size_dt_struct: fdt_header_le.size_dt_struct.to_be(),
        };

        const FDT_MAGIC: u32 = 0xd00dfeed;
        assert_eq!(fdt_header.magic, FDT_MAGIC, "Wrong FDT Magic Number");
        assert!(fdt_header.off_mem_rsvmap < fdt_header.off_dt_struct, "Offset not in order");
        assert!(fdt_header.off_dt_struct < fdt_header.off_dt_strings, "Offset not in order");
        assert!(fdt_header.size_dt_struct + fdt_header.size_dt_strings < fdt_header.total_size, "Size problem");
        assert!(fdt_header.off_dt_struct + fdt_header.size_dt_struct < fdt_header.total_size, "Size problem");
        assert!(fdt_header.off_dt_strings + fdt_header.size_dt_strings < fdt_header.total_size, "Size problem");

        let buf = unsafe {
            &*slice_from_raw_parts(dtb as *const u8, fdt_header.total_size as usize)
        };

        sbi_print_str("Fdt total size: ");
        sbi_println_number_base10(buf.len());

        let fdt = fdt::Fdt::new(buf).unwrap();

        sbi_print_str("Cpus: ");
        let cpus_number = fdt.cpus().count();
        sbi_println_number_base10(cpus_number);
    }

    // pub fn print_things(&self) {
    //     sbi_println_str("TEST DTB");
    //
    //     sbi_println_str("----------");
    //
    //     sbi_print_str("Offset Memory Reservation : ");
    //     sbi_println_number_base10(self.off_mem_rsvmap as usize);
    //
    //     sbi_print_str("Offset Structure Block : ");
    //     sbi_print_number_base10(self.off_dt_struct as usize);
    //     sbi_print_str(" - ");
    //     sbi_println_number_base10((self.off_dt_struct + self.size_dt_struct) as usize);
    //
    //     sbi_print_str("Offset Strings Block : ");
    //     sbi_print_number_base10(self.off_dt_strings as usize);
    //     sbi_print_str(" - ");
    //     sbi_println_number_base10((self.off_dt_strings + self.size_dt_strings) as usize);
    //
    //     sbi_println_str("----------");
    //
    //     let base_addr = ((self as *const FdtHeader) as *const u8) as usize;
    //
    //     // Test Memory Reservation Parsing
    //     sbi_println_str("Memory Reservation Block :");
    //
    //     let mut addr = unsafe {
    //         let addr = (self as *const FdtHeader) as *const u8;
    //         addr.byte_offset(self.off_mem_rsvmap as isize)
    //     };
    //
    //     let mut entry: &FdtReserveEntry = unsafe {
    //         let e = &mut *(addr as *mut FdtReserveEntry);
    //         e.address = e.address.to_be();
    //         e.size = e.size.to_be();
    //         e
    //     };
    //
    //     while entry.address.to_be() != 0 && entry.size.to_be() != 0 {
    //         if addr as usize + core::mem::size_of::<FdtReserveEntry>() > base_addr + self.off_dt_struct as usize {
    //             sbi_println_str("!! Should not overlap with the Structure Block !!");
    //             sbi_println_number_base10(self.off_mem_rsvmap as usize + addr as usize + core::mem::size_of::<FdtReserveEntry>());
    //             sbi_println_number_base10(base_addr + self.off_dt_struct as usize);
    //             break;
    //         }
    //         sbi_print_str("Addr: ");
    //         // If I change entry.address -> entry.address.to_be() it crashes here
    //         sbi_println_number_base10(entry.address.to_be() as usize);
    //         // sbi_print_str("Size: ");
    //         // sbi_println_number_base10(entry.size as usize);
    //         sbi_println_str("");
    //         addr = unsafe { addr.byte_offset(core::mem::size_of::<FdtReserveEntry>() as isize) };
    //         // entry = unsafe { &*(addr as *const FdtReserveEntry) };
    //         entry = unsafe {
    //             let e = &mut *(addr as *mut FdtReserveEntry);
    //             e.address = e.address.to_be();
    //             e.size = e.size.to_be();
    //             e
    //         };
    //     }
    //
    //     sbi_println_str("----------");
    //     // Test Structure Block Parsing
    //     sbi_println_str("Structure Block :");
    //
    //     let structure_block = unsafe {
    //         &*slice_from_raw_parts(((self as *const FdtHeader) as *const u32).byte_offset(self.off_dt_struct as isize), self.size_dt_struct as usize)
    //     };
    //
    //     // Should print a lot of token but do not
    //     for x in structure_block {
    //         match (*x).to_be() {
    //             FDT_BEGIN_NODE => sbi_println_str("BEGIN NODE"),
    //             FDT_END_NODE => sbi_println_str("END NODE"),
    //             FDT_PROP => sbi_println_str("PROP NODE"),
    //             FDT_NOP => sbi_println_str("NOP NODE"),
    //             FDT_END => sbi_println_str("END TREE"),
    //             _ => (),
    //         }
    //     }
    //
    //     sbi_println_str("END TEST DTB");
    // }
}
