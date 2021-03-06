#!/bin/sh

# You might need to change this...
ESP_QEMU_PATH=$HOME/Projects/3rd-party/espressif/qemu/build
IDF_PATH=$HOME/.espressif/esp-idf-v4.3
IDF_VERSION=v4.3
BUILD=debug

# use same esp-idf env as esp-idf-sys library
ESPTOOL_CMD="python3 $IDF_PATH/components/esptool_py/esptool/esptool.py"

TARGET=xtensa-esp32-espidf # Don't change this. Only the ESP32 chip is supported in QEMU for now

$ESPTOOL_CMD --chip esp32 elf2image target/$TARGET/$BUILD/mandelbrot-esp32
$ESPTOOL_CMD --chip esp32 merge_bin --output target/$TARGET/$BUILD/mandelbrot-esp32-qemu.bin --fill-flash-size 4MB 0x1000 qemu_bins/bootloader_$IDF_VERSION.bin  0x8000 qemu_bins/partitions_$IDF_VERSION.bin  0x10000 target/$TARGET/$BUILD/mandelbrot-esp32.bin --flash_mode dio --flash_freq 40m --flash_size 4MB
$ESP_QEMU_PATH/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=tcp:127.0.0.1:7888-:80 -drive file=target/$TARGET/$BUILD/mandelbrot-esp32-qemu.bin,if=mtd,format=raw -m 4M -global driver=timer.esp32.timg,property=wdt_disable,value=true

