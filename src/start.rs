extern "C" {
    static mut _start_bss: u8;
    static mut _end_bss: u8;

    static mut _start_data: u8;
    static mut _end_data: u8;
}

#[no_mangle] // This function must have the same name as in entry.S
pub unsafe extern "C" fn start(hart_id: usize, dtb: usize) -> ! {

    // Enable interrupts to supervisor level (external, timer, software)
    riscv::register::sie::set_sext(); // SEIE
    riscv::register::sie::set_stimer(); // STIE
    riscv::register::sie::set_ssoft(); // SSIE

    // Zeroing the .BSS and .DATA sections
    let count = &_end_bss as *const u8 as usize - &_start_bss as *const u8 as usize;
    core::ptr::write_bytes(&mut _start_bss as *mut u8, 0, count);

    let count = &_end_data as *const u8 as usize - &_start_data as *const u8 as usize;
    core::ptr::copy_nonoverlapping(&_start_data as *const u8, &mut _start_data as *mut u8, count);

    crate::main(hart_id, dtb);
}
