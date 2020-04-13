# Snake Game

Retro snake game for `x86_64` architecture machines without abstractions provided by OS. This is a fun mini project I implemented while learning about OS development from a series of wonderful [blog posts](https://os.phil-opp.com/) by Philipp Oppermann. Also, this was an attempt to learn more about Rust by implementing my second non-trivial Rust project.

## DEMO!
![Snake game GIF demo](https://raw.githubusercontent.com/analyst1001/snake_game/master/snake_game_demo.gif)

## Getting Started

Build the ISO file using `make iso` and run it in QEMU using `make run`.

### Pre-requisites

For building from source, you would need:
1. `cargo` configured with nightly Rust compiler to compile Rust source code. Tested with `rustc 1.43.0-nightly (58b834344 2020-02-05)`
2. `nasm` assembler to assemble assembly source code
3. `ld` linker to link compiled object files
4. `grub-mkrescue` and dependencies for building ISO file
5. `qemu-system-x86_64` for running game inside QEMU for `x86_64` architecture

### Building

Use `make` to build a binary game file. This could be linked with a custom bootloader (in progress), or packaged as part of ISO file using GRUB with `make iso`.

### Running Tests

Still TBD

## Acknowledgments
* Philipp Oppermann's wonderful [blog posts](https://os.phil-opp.com/)
* [OSDev Wiki](https://wiki.osdev.org/)
* Miscellaneous blog posts/articles introducing OS development
