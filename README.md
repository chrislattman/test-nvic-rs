# Raspberry Pi Pico 2 Rust NVIC example

This is essentially a no_std Rust version of https://github.com/chrislattman/test-nvic

It uses `rtic`, which employs Rust's safety features to make mathematical guarantees about interrupt priorities.

Instructions:

- Follow the instructions at https://github.com/chrislattman/test-bare-metal-pico to install `picotool` if you haven't already
- Run `rustup target add thumbv8m.main-none-eabihf`

To build application and run on board:

- Unplug USB cable from board
- Hold down BOOTSEL button while plugging in USB cable
- Run `cargo run --release`
