# MagicOs

Just for fun !

Inspirations from [xv6-riscv](https://github.com/mit-pdos/xv6-riscv), [rrxv6](https://github.com/yodalee/rrxv6) and [Writing an OS in Rust](https://os.phil-opp.com)

## Dependencies

In order to build and run the os from a Linux machine you'll need :
- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make) which you can install via `cargo install --force cargo-make`
- QEMU for Riscv :
  1. `git clone https://github.com/qemu/qemu` (or get it from [QEMU download page](https://download.qemu.org))
  2. `cd qemu`
  3. *(if downloaded from GitHub)* `git checkout stable-6.1` (chose a recent version you want)
  4. `./configure --target-list=riscv64-softmmu`
  5. `make`
  6. `sudo make install`
- *(if you want to use gdb on your machine)* The Newlib cross-compiler of [Riscv-GNU-Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)

## Build & Run

To run the OS with QEMU just do `cargo make qemu`

## Debugging with gdb

Run in a terminal `cargo make qemu-gdb`
And in another terminal (in this directory) `riscv64-unknown-elf-gdb`

## Done

- [x] Parse the DTB (using a library)
- [x] Use the `linked-list-crate` as the allocator

## Todo

- [ ] Create the page tables
- [ ] Create my own allocator
