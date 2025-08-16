#!/bin/bash

# Create the disk for qemu to use
dd if=/dev/zero of=hdd.dsk count=32 bs=1M
