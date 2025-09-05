#!/bin/bash

# install the prereqs for building the OS from source

rustup toolchain install nightly-2025-07-19-x86_64-unknown-linux-gnu
rustup default nightly-2025-07-19-x86_64-unknown-linux-gnu
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add rust-analyzer

# The following only work on debian based systems, get riscv64 from qemu and
# a riscv64 c++ linux cross compiler for your distribution
sudo apt-get -y install g++-riscv64-linux-gnu
sudo apt install qemu-system-riscv64
