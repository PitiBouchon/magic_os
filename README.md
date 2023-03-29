# MagicOs

Just for fun !

Inspirations from [xv6-riscv](https://github.com/mit-pdos/xv6-riscv), [rrxv6](https://github.com/yodalee/rrxv6) and [Writing an OS in Rust](https://os.phil-opp.com)

## Dependencies

In order to build and run the os from a Linux machine you'll need :
- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make) which you can install via `cargo install --force cargo-make`
- The Newlib cross-compiler of [Riscv-GNU-Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- QEMU for Riscv :
  1. `git clone https://github.com/qemu/qemu` (or get it from [QEMU download page](https://download.qemu.org))
  2. `cd qemu`
  3. *(if downloaded from GitHub)* `git checkout stable-6.1` (chose a recent version you want)
  4. `./configure --target-list=riscv64-softmmu`
  5. `make`
  6. `sudo make install`
- *If you didn't clone the repo with opensbi just pull the git submodule with* `git submodule update --init --recursive`

## Build & Run

To run the OS with QEMU just do `cargo make qemu`


## Debugging with gdb

Run in a terminal `cargo make qemu-gdb`
And in another terminal (in this directory) `riscv64-unknown-elf-gdb`

## Issues

- If I add :
```toml
[profile.dev]
panic = "abort"
```
I get strange print

- Memory Reservation does not end well
- Structure Block data is wrong

## Todo

- [ ] Parse the DTB
- [ ] Create the allocator
- [ ] Create the page tables
