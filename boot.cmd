setenv bootdelay '3'
setenv bootargs ""
saveenv
ext4load mmc 0:1 0x40200000 /boot/os.elf
bootelf 0x40200000
