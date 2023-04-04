use crate::sbi_print::{
    sbi_print_number_base10, sbi_print_str, sbi_println_number_base10, sbi_println_str,
};

pub unsafe fn init_dtb(dtb: usize) {
    sbi_print_str("DTB at : ");
    sbi_println_number_base10(dtb);

    let fdt = fdt::Fdt::from_ptr(dtb as *const u8).unwrap();
    sbi_print_str("CPUs : ");
    sbi_println_number_base10(fdt.cpus().count());
}
