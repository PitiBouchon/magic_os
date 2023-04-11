fn main() {
    // Rebuild if the linked script has changed
    println!("cargo:rerun-if-changed=src/linker/linker.ld");

    // Rebuild if assembly target changed
    println!("cargo:rerun-if-changed=src/asm/entry.S");
    println!("cargo:rerun-if-changed=src/asm/kernelvec.S");
}
