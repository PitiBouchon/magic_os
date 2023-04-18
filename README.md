# MagicOs

Just for fun !

Inspirations from [xv6-riscv](https://github.com/mit-pdos/xv6-riscv), [rrxv6](https://github.com/yodalee/rrxv6) and [Writing an OS in Rust](https://os.phil-opp.com)

## Dependencies

In order to build and run the os from a Linux machine you'll need :
- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make) which you can install via `cargo install --force cargo-make`
- [QEMU](https://www.qemu.org/download/) for Riscv
- *(for linux, if you want to use gdb on your machine)* The Newlib cross-compiler of [Riscv-GNU-Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)

## Build & Run

To run the OS with QEMU just do `cargo make qemu`

## Debugging with gdb

Run in a terminal `cargo make qemu-gdb`
And in another terminal (in this directory) `riscv64-unknown-elf-gdb`

## Done

- [x] Parse the DTB (using a library)
- [x] Create the page tables
- [x] Create my own allocator (could be better)

## Todo

- [ ] Add disk drive support
- [ ] Add processes and scheduler
- [ ] Add user programs

