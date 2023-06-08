mod display_interface_impl;
mod sealed;
mod timing;

use core::marker::PhantomData;

pub use self::timing::{AccessMode, Timing};

use crate::pac;

/// A sub-bank of bank 1, with its own chip select output
pub trait SubBank: sealed::SealedSubBank {}
/// Sub-bank 1
pub struct SubBank1(());
impl sealed::SealedSubBank for SubBank1 {
    const BASE_ADDRESS: usize = 0x6000_0000;
}
impl SubBank for SubBank1 {}

/// An FMC configured as an LCD interface
pub struct FmcLcd;

impl FmcLcd {
    pub fn new(read_timing: &Timing, write_timing: &Timing) -> Lcd<SubBank1> {
        let fmc = unsafe { &*pac::FMC::ptr() };
        let rcc = unsafe { &*pac::RCC::ptr() };
        let gpiod = unsafe { &*pac::GPIOD::ptr() };
        let gpioe = unsafe { &*pac::GPIOE::ptr() };

        rcc.ahb1enr
            .modify(|_, w| w.gpioden().enabled().gpioeen().enabled());

        gpiod.afrl.modify(|_, w| {
            w.afrl0()
                .af12()
                .afrl1()
                .af12()
                .afrl4()
                .af12()
                .afrl5()
                .af12()
                .afrl7()
                .af12()
        });
        gpiod.afrh.modify(|_, w| {
            w.afrh8()
                .af12()
                .afrh9()
                .af12()
                .afrh10()
                .af12()
                .afrh11()
                .af12()
                .afrh14()
                .af12()
                .afrh15()
                .af12()
        });

        gpioe.afrl.modify(|_, w| w.afrl7().af12());
        gpioe.afrh.modify(|_, w| {
            w.afrh8()
                .af12()
                .afrh9()
                .af12()
                .afrh10()
                .af12()
                .afrh11()
                .af12()
                .afrh12()
                .af12()
                .afrh13()
                .af12()
                .afrh14()
                .af12()
                .afrh15()
                .af12()
        });

        // Enable the FMC
        rcc.ahb3enr.modify(|_, w| w.fmcen().enabled());
        rcc.ahb3rstr.modify(|_, w| w.fmcrst().reset());
        rcc.ahb3rstr.modify(|_, w| w.fmcrst().clear_bit());

        // Configure memory type and basic interface settings
        // The reference manuals are sometimes unclear on the distinction between banks
        // and sub-banks of bank 1. This driver uses addresses in the different sub-banks of
        // bank 1. The configuration registers for "bank x" (like FMC_BCRx) actually refer to
        // sub-banks, not banks. We need to configure and enable all four of them.
        configure_bcr1(&fmc.bcr1);
        configure_bcr(&fmc.bcr2);
        configure_bcr(&fmc.bcr3);
        configure_bcr(&fmc.bcr4);
        configure_btr(&fmc.btr1, read_timing);
        configure_btr(&fmc.btr2, read_timing);
        configure_btr(&fmc.btr3, read_timing);
        configure_btr(&fmc.btr4, read_timing);
        configure_bwtr(&fmc.bwtr1, write_timing);
        configure_bwtr(&fmc.bwtr2, write_timing);
        configure_bwtr(&fmc.bwtr3, write_timing);
        configure_bwtr(&fmc.bwtr4, write_timing);

        Lcd { _sub_bank: core::marker::PhantomData }
    }
}

/// Configures an SRAM/NOR-Flash chip-select control register for LCD interface use
fn configure_bcr1(bcr: &pac::fmc::BCR1) {
    bcr.write(|w| {
        w
            // The write fifo and WFDIS bit are missing from some models.
            // Where present, the FIFO is enabled by default.
            // ------------
            // Disable synchronous writes
            .cburstrw()
            .disabled()
            // Don't split burst transactions (doesn't matter for LCD mode)
            .cpsize()
            .no_burst_split()
            // Ignore wait signal (asynchronous mode)
            .asyncwait()
            .disabled()
            // Enable extended mode, for different read and write timings
            .extmod()
            .enabled()
            // Ignore wait signal (synchronous mode)
            .waiten()
            .disabled()
            // Allow write operations
            .wren()
            .enabled()
            // Default wait timing
            .waitcfg()
            .before_wait_state()
            // Default wait polarity
            .waitpol()
            .active_low()
            // Disable burst reads
            .bursten()
            .disabled()
            // Enable NOR flash operations
            .faccen()
            .enabled()
            // 16-bit bus width
            .mwid()
            .bits16()
            // NOR flash mode (compatible with LCD controllers)
            .mtyp()
            .flash()
            // Address and data not multiplexed
            .muxen()
            .disabled()
            // Enable this memory bank
            .mbken()
            .enabled()
    })
}

/// Configures an SRAM/NOR-Flash chip-select control register for LCD interface use
///
/// This is equivalent to `configure_bcr1`, but without the `WFDIS` and `CCLKEN` bits that are
/// present in BCR1 only.
fn configure_bcr(bcr: &pac::fmc::BCR) {
    bcr.write(|w| {
        w
            // Disable synchronous writes
            .cburstrw()
            .disabled()
            // Don't split burst transactions (doesn't matter for LCD mode)
            .cpsize()
            .no_burst_split()
            // Ignore wait signal (asynchronous mode)
            .asyncwait()
            .disabled()
            // Enable extended mode, for different read and write timings
            .extmod()
            .enabled()
            // Ignore wait signal (synchronous mode)
            .waiten()
            .disabled()
            // Allow write operations
            .wren()
            .enabled()
            // Default wait timing
            .waitcfg()
            .before_wait_state()
            // Default wait polarity
            .waitpol()
            .active_low()
            // Disable burst reads
            .bursten()
            .disabled()
            // Enable NOR flash operations
            .faccen()
            .enabled()
            // 16-bit bus width
            .mwid()
            .bits16()
            // NOR flash mode (compatible with LCD controllers)
            .mtyp()
            .flash()
            // Address and data not multiplexed
            .muxen()
            .disabled()
            // Enable this memory bank
            .mbken()
            .enabled()
    })
}

/// Configures a read timing register
fn configure_btr(btr: &pac::fmc::BTR, read_timing: &Timing) {
    btr.write(|w| unsafe {
        w.accmod()
            .variant(read_timing.access_mode.as_read_variant())
            .busturn()
            .bits(read_timing.bus_turnaround)
            .datast()
            .bits(read_timing.data)
            .addhld()
            .bits(read_timing.address_hold)
            .addset()
            .bits(read_timing.address_setup)
    })
}
/// Configures a write timing register
fn configure_bwtr(bwtr: &pac::fmc::BWTR, write_timing: &Timing) {
    bwtr.write(|w| unsafe {
        w.accmod()
            .variant(write_timing.access_mode.as_write_variant())
            .busturn()
            .bits(write_timing.bus_turnaround)
            .datast()
            .bits(write_timing.data)
            .addhld()
            .bits(write_timing.address_hold)
            .addset()
            .bits(write_timing.address_setup)
    })
}

/// An interface to an LCD controller using one sub-bank
///
/// This struct provides low-level read and write commands that can be used to implement
/// drivers for LCD controllers. Each function corresponds to exactly one transaction on the bus.
pub struct Lcd<S> {
    /// Phantom S
    ///
    /// S determines the chip select signal to use, and the addresses used with that signal.
    _sub_bank: PhantomData<S>,
}

impl<S> Lcd<S>
where
    S: SubBank,
{
    /// Writes a value with the data/command (address) signals set high
    pub fn write_data(&mut self, value: u16) {
        unsafe {
            core::ptr::write_volatile(S::DATA_ADDRESS as *mut u16, value);
        }
    }

    /// Writes a value with the data/command (address) signals set low
    pub fn write_command(&mut self, value: u16) {
        unsafe {
            core::ptr::write_volatile(S::COMMAND_ADDRESS as *mut u16, value);
        }
    }

    /// Reads a value with the data/command (address) signals set high
    pub fn read_data(&self) -> u16 {
        unsafe { core::ptr::read_volatile(S::DATA_ADDRESS as *const u16) }
    }

    /// Reads a value with the data/command (address) signals set low
    pub fn read_command(&self) -> u16 {
        unsafe { core::ptr::read_volatile(S::COMMAND_ADDRESS as *const u16) }
    }
}
