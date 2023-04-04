use crate::sbi_print::{sbi_print_number_base10, sbi_print_str, sbi_println_number_base10, sbi_println_str};

pub unsafe fn init_dtb(dtb: usize) {
    sbi_print_str("DTB at : ");
    sbi_println_number_base10(dtb);

    match fdt::Fdt::from_ptr(dtb as *const u8) {
        Ok(_) => sbi_println_str("OK"),
        Err(fdt::FdtError::BadPtr) => sbi_println_str("BadPtr"),
        _ => sbi_println_str("unknown error"),
    }
}
