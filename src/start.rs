core::arch::global_asm!(include_str!("asm/entry.S"));

extern "C" {
    static mut _start_bss: u8;
    static mut _end_bss: u8;
}

#[no_mangle] // This function must have the same name as in entry.S
pub unsafe extern "C" fn start(hart_id: usize, dtb: usize) -> ! {
    // Zeroing the .BSS section (which correspond to uninitialized values)
    let bss_size = &_end_bss as *const u8 as usize - &_start_bss as *const u8 as usize;
    core::ptr::write_bytes(&mut _start_bss as *mut u8, 0, bss_size);

    crate::main(hart_id, dtb);
}
