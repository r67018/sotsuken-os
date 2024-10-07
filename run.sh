#!/bin/bash

EDK2_DIR=~/edk2
BOOTLOADER_DIR=$EDK2_DIR/Build/MikanLoaderX64/DEBUG_CLANG38/X64/MikanLoaderPkg/Loader/DEBUG/Loader.efi
OSBOOK_DIR=~/osbook
KERNEL_FILE=./kernel/kernel.elf

$OSBOOK_DIR/devenv/run_qemu.sh $BOOTLOADER_DIR $KERNEL_FILE

