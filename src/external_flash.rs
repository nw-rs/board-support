#![allow(dead_code)]
pub const FLASH_START: u32 = 0x90000000;
pub const FLASH_END: u32 = FLASH_START + FLASH_SIZE;

use core::marker::PhantomData;

use crate::pac::{GPIOB, GPIOC, GPIOD, GPIOE, QUADSPI, RCC};
use cortex_m::asm;

// 2^23 = 8MB
const FLASH_ADDRESS_SIZE: u8 = 23;
const ADDRESS_WIDTH: u8 = 3;
const FLASH_SIZE: u32 = 8388608;

const N_4K_SECTORS: u8 = 8;
const N_32K_SECTORS: u8 = 1;
const N_64K_SECTORS: u8 = 127;
const N_SECTORS: u8 = N_4K_SECTORS + N_32K_SECTORS + N_64K_SECTORS;
const ADDRESS_BITS_4K: u8 = 12;
const ADDRESS_BITS_32K: u8 = 15;
const ADDRESS_BITS_64K: u8 = 16;
const PAGE_SIZE: usize = 256;

//#[allow(dead_code)]
#[repr(u8)]
enum QspiWidth {
    None = 0b00,
    Single = 0b01,
    Dual = 0b10,
    Quad = 0b11,
}

/// The different QSPI functional modes.
#[repr(u8)]
enum QspiMode {
    IndirectWrite = 0b00,
    IndirectRead = 0b01,
    AutoPolling = 0b10,
    MemoryMapped = 0b11,
}

/// The number of bytes required to specify addresses on the chip.
#[repr(u8)]
enum QspiSize {
    OneByte = 0b00,
    TwoBytes = 0b01,
    ThreeBytes = 0b10,
    FourBytes = 0b11,
}

/// Commands (instructions) that can be sent to the flash chip.
#[repr(u8)]
pub enum Command {
    ReadStatusRegister1 = 0x05,
    ReadStatusRegister2 = 0x35,
    WriteStatusRegister = 0x01,
    WriteStatusRegister2 = 0x31,
    WriteEnable = 0x06,
    WriteEnableVolatile = 0x50,
    ReadData = 0x03,
    FastRead = 0x0B,
    FastReadQuadIO = 0xEB,
    PageProgram = 0x02,
    QuadPageProgram = 0x33,
    EnableQPI = 0x38,
    EnableReset = 0x66,
    Reset = 0x99,
    ChipErase = 0xC7,
    Erase4KbyteBlock = 0x20,
    Erase32KbyteBlock = 0x52,
    Erase64KbyteBlock = 0xD8,
    SetReadParameters = 0xC0,
    DeepPowerDown = 0xB9,
    ReleaseDeepPowerDown = 0xAB,
    ReadIds = 0x90,
    ReadJEDECID = 0x9F,
}

#[derive(Copy, Clone, Debug)]
pub enum Uninitialized {}
#[derive(Copy, Clone, Debug)]
pub enum Indirect {}
#[derive(Copy, Clone, Debug)]
pub enum MemoryMapped {}

#[derive(Copy, Clone, Debug)]
pub struct ExternalFlash<MODE> {
    mode: PhantomData<MODE>,
}

impl ExternalFlash<Uninitialized> {
    pub fn new() -> Self {
        let rcc = unsafe { &*RCC::PTR };
        let qspi = unsafe { &*QUADSPI::PTR };

        let gpiob = unsafe { &*GPIOB::PTR };
        let gpioc = unsafe { &*GPIOC::PTR };
        let gpiod = unsafe { &*GPIOD::PTR };
        let gpioe = unsafe { &*GPIOE::PTR };

        gpiob.afrl.modify(|_, w| w.afrl2().af9().afrl6().af10());
        gpioc.afrh.modify(|_, w| w.afrh9().af9());
        gpiod.afrh.modify(|_, w| w.afrh12().af9().afrh13().af9());
        gpioe.afrl.modify(|_, w| w.afrl2().af9());

        gpiob
            .moder
            .modify(|_, w| w.moder2().alternate().moder6().alternate());
        gpioc.moder.modify(|_, w| w.moder9().alternate());
        gpiod
            .moder
            .modify(|_, w| w.moder12().alternate().moder13().alternate());
        gpioe.moder.modify(|_, w| w.moder2().alternate());

        gpiob
            .ospeedr
            .modify(|_, w| w.ospeedr2().very_high_speed().ospeedr6().very_high_speed());
        gpioc.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());
        gpiod.ospeedr.modify(|_, w| {
            w.ospeedr12()
                .very_high_speed()
                .ospeedr13()
                .very_high_speed()
        });
        gpioe.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

        rcc.ahb3enr.modify(|_, w| w.qspien().set_bit());

        rcc.ahb3rstr.modify(|_, w| w.qspirst().reset());
        rcc.ahb3rstr.modify(|_, w| w.qspirst().clear_bit());

        // Configure controller for flash chip.
        unsafe {
            qspi.dcr.write_with_zero(|w| {
                w.fsize()
                    .bits(FLASH_ADDRESS_SIZE - 1)
                    .csht()
                    .bits(2)
                    .ckmode()
                    .set_bit()
            });
            qspi.cr
                .write_with_zero(|w| w.prescaler().bits(3).en().set_bit());
        }

        Self { mode: PhantomData }
    }

    /// Turns on the chip and tells it to switch to QPI mode.
    #[must_use]
    pub fn init(self, ahb_frequency: u32) -> ExternalFlash<Indirect> {
        let mut delay = cortex_m::delay::Delay::new(
            unsafe { cortex_m::Peripherals::steal().SYST },
            ahb_frequency,
        );

        // Turn on the chip.
        self.send_spi_command(Command::ReleaseDeepPowerDown, None);
        delay.delay_us(3_u32);

        // Enable writing to the chip so that the status register can be changed.
        self.send_spi_command(Command::WriteEnableVolatile, None);

        // Set QE in the chip's status register.
        self.send_spi_command(Command::WriteStatusRegister2, Some(0x02));

        let qspi = unsafe { &*QUADSPI::PTR };

        while qspi.sr.read().busy().bit_is_set() {}

        let qspi = ExternalFlash { mode: PhantomData };

        qspi
    }

    /// Sends a command with optional data in SPI mode.
    fn send_spi_command(&self, command: Command, data: Option<u8>) {
        let qspi = unsafe { &*QUADSPI::PTR };
        qspi.dlr.reset();

        if let Some(data) = data {
            qspi.abr.write(|w| unsafe { w.bits(u32::from(data)) });
        }

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .instruction()
                .bits(command as u8);

            if data.is_some() {
                w.abmode()
                    .bits(QspiWidth::Single as u8)
                    .absize()
                    .bits(QspiSize::OneByte as u8);
            }

            w
        });

        while qspi.sr.read().busy().bit_is_set() {}
    }
}

impl ExternalFlash<Indirect> {
    /// Reads the manufacturer and device IDs.
    ///
    /// The first value is the manufacturer ID and the second one it the device ID.
    pub fn read_ids(&self) -> (u8, u8) {
        let qspi = unsafe { &*QUADSPI::PTR };

        qspi.dlr.write(|w| unsafe { w.dl().bits(2 - 1) });
        qspi.ar.reset();

        // The STM32 doesn't seem to release the QSPI pins early enough, after the
        // address is transmitted. The short bus contention leads to invalid data on
        // the rising clock edge. Using a later sampling point fixes this problem.
        // TODO: can the bus contention be eliminated entirely?
        qspi.cr.modify(|_, w| w.sshift().set_bit());

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(Command::ReadIds as u8)
        });

        qspi.ar.reset();

        let data = qspi.dr.read().bits();

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        qspi.cr.modify(|_, w| w.sshift().clear_bit());

        (data as u8, (data >> 8) as u8)
    }

    /// Reads status register 1.
    pub fn read_status_register1(&self) -> u8 {
        self.read_status_register(Command::ReadStatusRegister1)
    }

    /// Reads status register 2.
    pub fn read_status_register2(&self) -> u8 {
        self.read_status_register(Command::ReadStatusRegister2)
    }

    fn read_status_register(&self, command: Command) -> u8 {
        let qspi = unsafe { &*QUADSPI::PTR };
        qspi.dlr.write(|w| unsafe { w.dl().bits(1 - 1) });

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .instruction()
                .bits(command as u8)
        });

        let data = qspi.dr.read().bits();

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        data as u8
    }

    /// Reads bytes until the buffer is full.
    pub fn read_bytes(&self, address: u32, buffer: &mut [u8]) {
        let qspi = unsafe { &*QUADSPI::PTR };
        qspi.dlr.write(|w| unsafe { w.dl().bits(1 - 1) });

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectRead as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .dcyc()
                .bits(6)
                .instruction()
                .bits(Command::FastRead as u8)
        });

        qspi.ar.write(|w| unsafe { w.bits(address) });

        for d in buffer {
            *d = qspi.dr.read().bits() as u8;
        }

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }

    /// Programs a page.
    pub fn program_page(&self, mut address: u32, data: &[u8]) {
        let qspi = unsafe { &*QUADSPI::PTR };

        let offset: u8 = (address & (PAGE_SIZE - 1) as u32) as u8;
        let mut fits_in_page: usize = PAGE_SIZE - offset as usize;
        let mut length = data.len();
        let mut start = 0;
        while length > 0 {
            if fits_in_page > length {
                fits_in_page = length;
            }

            self.write_enable();

            qspi.dlr
                .write(|w| unsafe { w.dl().bits(data.len() as u32 - 1) });

            qspi.ccr.write(|w| unsafe {
                w.fmode()
                    .bits(QspiMode::IndirectWrite as u8)
                    .imode()
                    .bits(QspiWidth::Single as u8)
                    .dmode()
                    .bits(QspiWidth::Quad as u8)
                    .admode()
                    .bits(QspiWidth::Quad as u8)
                    .adsize()
                    .bits(QspiSize::ThreeBytes as u8)
                    .instruction()
                    .bits(Command::PageProgram as u8)
            });

            qspi.ar.write(|w| unsafe { w.bits(address) });

            for byte in &data[start..(start + fits_in_page)] {
                // while qspi.sr.read().ftf().bit_is_clear() {
                //     asm::nop();
                // }
                unsafe {
                    core::ptr::write_volatile(&qspi.dr as *const _ as *mut u8, *byte);
                }
            }

            length -= fits_in_page;
            address += fits_in_page as u32;
            start += fits_in_page;
            fits_in_page = PAGE_SIZE;

            while qspi.sr.read().busy().bit_is_set() {
                asm::nop();
            }
        }

        self.wait_busy();
    }

    /// Enables writing.
    pub fn write_enable(&self) {
        self.command(Command::WriteEnable);
    }

    /// Disables writing.
    pub fn write_disable(&self) {
        self.command(Command::WriteEnable);
    }

    /// Erases the chip.
    pub fn chip_erase(&self) {
        self.command(Command::ChipErase);
        self.wait_busy();
    }

    pub fn block_erase_4k(&self, address: u32) {
        let qspi = unsafe { &*QUADSPI::PTR };

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .instruction()
                .bits(Command::Erase4KbyteBlock as u8)
        });

        qspi.ar.write(|w| unsafe { w.bits(address) });

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }

        self.wait_busy();
    }

    fn command(&self, command: Command) {
        let qspi = unsafe { &*QUADSPI::PTR };

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .instruction()
                .bits(command as u8)
        });

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }

    /// Waits until the busy flag is cleared.
    fn wait_busy(&self) {
        while self.read_status_register1() & 0x01 != 0 {
            asm::nop();
        }
    }

    fn set_read_parameters(&self) {
        let qspi = unsafe { &*QUADSPI::PTR };

        // 104Mhz -> 6 dummy clocks, 8-byte wrap
        qspi.abr.write(|w| unsafe { w.bits(0b0010_0000) });

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::IndirectWrite as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .abmode()
                .bits(QspiWidth::Quad as u8)
                .absize()
                .bits(QspiSize::OneByte as u8)
                .instruction()
                .bits(Command::SetReadParameters as u8)
        });

        while qspi.sr.read().busy().bit_is_set() {
            asm::nop();
        }
    }

    pub fn into_memory_mapped(self) -> ExternalFlash<MemoryMapped> {
        let qspi = unsafe { &*QUADSPI::PTR };

        qspi.abr.write(|w| unsafe { w.bits(0) });

        qspi.ccr.write(|w| unsafe {
            w.fmode()
                .bits(QspiMode::MemoryMapped as u8)
                .dmode()
                .bits(QspiWidth::Quad as u8)
                .dcyc()
                .bits(4)
                .abmode()
                .bits(QspiWidth::Quad as u8)
                .absize()
                .bits(0)
                .admode()
                .bits(QspiWidth::Quad as u8)
                .adsize()
                .bits(QspiSize::ThreeBytes as u8)
                .imode()
                .bits(QspiWidth::Single as u8)
                .instruction()
                .bits(Command::FastReadQuadIO as u8)
                .sioo()
                .set_bit()
        });

        ExternalFlash { mode: PhantomData }
    }
}
