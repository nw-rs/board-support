# NW Board Support

Board support crate for the Numworks n0110 calculator.

Provides easy access to the display, key pad, LED and usb port built into the
calculator. Also provides functions to quickly initialize the clocks and MPU.
This crate uses several unsafe functions to avoid requiring ownership of the
peripherals. This crate is currently unstable and under development and
breaking changes may occur without warning.

Currently the only major feature that is missing is a driver for the external
flash chip that comes with the calculator.

An example implementation of this crate can be seen in
[`nw-rs/bootloader`](https://github.com/nw-rs/bootloader).
