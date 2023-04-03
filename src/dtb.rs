use core::ptr::slice_from_raw_parts;
use crate::sbi_print::{sbi_print_number_base10, sbi_print_str, sbi_println_number_base10, sbi_println_str};

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
    pub unsafe fn init_dtb(dtb: usize) {
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
}
