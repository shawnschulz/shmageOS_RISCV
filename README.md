# shmageOS

One day shmageOS will be an experimental operating system using a partitioned global address space memory model for parallel programming and a pooled distributed database file system. For now though, it's just me fighting with the linker and being really confused at the RISCV manual.

Many parts getting started are adapted from Stephen Marz's excellent tutorial on building a RISCV operating system in rust: https://osblog.stephenmarz.com/


## testing it out

You'll need the riscv64-linux-gnu-g++ cross compiler (or an equally capable cross compiler on your system). Right now for testing it runs on qemu, but i'd like to get a u-boot loaded image working on some real hardware soon. You'll need qemu-system-riscv64 as well.

Clone the repo:

``` sh
git clone git@github.com:shawnschulz/shmageOS_RISCV.git
cd shmageOS_RISCV
```

If you have apt, you can use the apt repos to install the neede dependencies.

``` sh
# you can use the apt repositories on debian/ubuntu
sudo apt install riscv64-linux-gnu-g++
sudo apt install qemu-system-riscv64
```

Once you've got those installed, qemu is going to look for a disk image file called "hdd.dsk". If you have bash you can use the shell script (prob sh will work too)

``` sh
bash make_disk.sh
```

Then just run make run

``` sh
make run
```

You'll enter a very barebones stack allocated command prompt for the shell. You can try running the unit tests:


``` shmage 
t(-_-) — ˎˊ˗test
```
