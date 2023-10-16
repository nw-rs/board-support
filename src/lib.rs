#![no_std]

#[cfg(feature = "n0110")]
pub use stm32f7::stm32f730 as pac;

pub mod clocks;

pub mod n0120;

#[cfg(feature = "dfu")]
pub mod dfu;

#[cfg(feature = "display")]
pub mod display;

#[cfg(feature = "external_flash")]
pub mod external_flash;

#[cfg(feature = "display")]
pub mod fmc_lcd;

#[cfg(feature = "keypad")]
pub mod keypad;

pub mod led;

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(test)]
mod test {}
