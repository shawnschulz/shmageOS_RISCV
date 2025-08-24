#!/usr/bin/env sh

# Flashes the sd card at the

# sudo umount /dev/sda
sudo dd if=shmageos.bin of=/dev/sdb1 bs=512 seek=2048 && sync
