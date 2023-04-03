use std::process::Command;
use cc::Build;

fn main() -> Result<(), ()> {
    // Rebuild if the linked script has changed
    println!("cargo:rerun-if-changed=src/linker/linker.ld");

    // Rebuild if assembly target changed
    println!("cargo:rerun-if-changed=src/asm/entry.S");
    println!("cargo:rerun-if-changed=src/asm/kernelvec.S");

    // Compile assembly files
    Build::new()
        .debug(true)
        .file("src/asm/entry.S")
        .file("src/asm/kernelvec.S")
        .compiler("/opt/riscv/bin/riscv64-unknown-elf-gcc")
        .compile("asm");

    // Compile OpenSBI
    if !Command::new("make")
        .current_dir("opensbi")
        .env("CROSS_COMPILE", "riscv64-unknown-elf-")
        .arg("PLATFORM=generic")
        .arg("FW_JUMP=y")
        .arg("FW_JUMP_ADDR=0x80200000") // Must be the same as in linker.ld
        .status()
        .unwrap()
        .success() {
        return Err(());
    }

    Ok(())
}
