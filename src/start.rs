#[no_mangle] // This function must have the same name as in entry.S
pub unsafe extern "C" fn start(hart_id: usize, dtb: usize) -> ! {

    // Enable interrupts to supervisor level (external, timer, software)
    riscv::register::sie::set_sext(); // SEIE
    riscv::register::sie::set_stimer(); // STIE
    riscv::register::sie::set_ssoft(); // SSIE

    crate::main(hart_id, dtb);
}
