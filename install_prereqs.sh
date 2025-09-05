#!/bin/bash

# install the prereqs for building the OS from source

rustup toolchain install nightly-2025-07-19-x86_64-unknown-linux-gnu
rustup default nightly-2025-07-19-x86_64-unknown-linux-gnu
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add rust-analyzer
