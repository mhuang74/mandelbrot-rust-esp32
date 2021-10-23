# mandelbrot-rust-esp32

Render Mandelbrot Images via Rust on ESP32.

Based on [rust-esp32-std-hello](https://github.com/ivmarkov/rust-esp32-std-hello)

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
