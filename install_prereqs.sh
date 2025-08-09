#!/bin/bash

# install the prereqs for building the OS from source

rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils