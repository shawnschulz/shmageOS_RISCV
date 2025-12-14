# let's try just manually setting this in the uboot environment,
# since the boot.scr is not working
setenv bootcmd 'ext4load mmc 0:1 0x11000000 /boot/kernel.bin ; go 0x11000000'
saveenv
ext4load mmc 0:1 0x11000000 /boot/kernel.bin
go 0x11000000

# ext4load mmc 0:1 0x11000000 /boot/os.elf
# load mmc 0:1 0x200000 /boot/os.elf
# bootelf -p 0x11000000
