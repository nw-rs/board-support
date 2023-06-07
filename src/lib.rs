#![no_std]

pub use stm32f7::stm32f730 as pac;

pub mod clocks;

#[cfg(feature = "dfu")]
pub mod dfu;

pub mod external_flash;

pub mod keypad;

pub mod led;

#[cfg(feature = "usb")]
pub mod usb;

mod test {}
