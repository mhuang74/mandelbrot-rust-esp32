# mandelbrot-rust-esp32

Render Mandelbrot Images via Rust on ESP32.

Based on [rust-esp32-std-demo](https://github.com/ivmarkov/rust-esp32-std-demo)

## Build

- Install the [Rust Espressif compiler fork and the Espressif LLVM Clang fork](https://github.com/esp-rs/rust) 
  - clean up old esp toolchain via `rm -rf ~/.rustup/toolchains/esp`
  - install latest esp toolchain via this [script](https://github.com/esp-rs/rust-build/blob/main/install-rust-toolchain.sh)
  - make sure to add IDF_TOOL_XTENSA_ELF_CLANG to PATH and LIBPATH
- The build uses `ldproxy` linker wrapper from [embuild](https://crates.io/crates/embuild), so install [ldproxy](https://crates.io/crates/embuild/ldproxy):
  - `cargo install ldproxy`
- Set Wifi credentials via `~/.cargo/config.toml`

```
[env]
RUST_ESP32_STD_HELLO_WIFI_SSID = "SSID"
RUST_ESP32_STD_HELLO_WIFI_PASS = "PASSWORD"
```
- Make sure correct `target` is set in `./.cargo/config.toml`
- Build: `cargo build` or `cargo build --release`
- Sometimes need to wipe out old platformio so toolchain can install latest
  - `rm -rf ~/.platformio`

## Flash

- install command-line espflash
  - `cargo install espflash`
- `espflash /dev/ttyUSB0 target/xtensa-esp32-espidf/debug/mandelbrot-esp32`

## Monitor
- install miniterm from [pySerial](https://pyserial.readthedocs.io/en/latest/pyserial.html)
  - `apt-get install python-serial`
- `miniterm --raw /dev/ttyUSB0 115200`
  - sometimes need press reset button on ESP32 to see full logs from bootup
  - `ctrl-]` to exit

## TODO 

- [X] stripped down version of rust-esp32-std-hello as base
- [X] add `num` dependency
  - cannot compile `num=0.4` crate, hitting this [error](https://github.com/espressif/llvm-project/issues/13)
    - FIXED: add `lto = "thin"` to dev and release profiles of Cargo.toml 
- [X] add `image` dependency
  - cannot compile `image=0.23`, hitting Atomic64 error from crossbeam=0.8.5
    - FIXED: lower to `image=0.8`
- [ ] add `rayon` dependency 
  - cannot compile any version of rayon due to Atomic64 error from crossbeam
- [X] switch to use `crossbeam=0.7` instead and follow the _task-queue_ approach from Programming Rust 
