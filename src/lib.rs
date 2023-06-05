#![no_std]

use cortex_m::peripheral::MPU;

pub use stm32f7::stm32f730 as pac;

pub use clocks::init_clocks;

pub mod clocks;
pub mod external_flash;

pub const HCLK: u32 = 192_000_000;

pub fn init_mpu() {
    cortex_m::asm::dmb();

    unsafe {
        const FULL_ACCESS: u32 = 0b011 << 24;
        const SIZE_512MB: u32 = 28 << 1;
        const SIZE_8MB: u32 = 22 << 1;
        const DEVICE_SHARED: u32 = 0b000001 << 16;
        const NORMAL_SHARED: u32 = 0b000110 << 16;

        let mpu = &*MPU::PTR;

        mpu.ctrl.write(0);

        // Flash
        mpu.rnr.write(0);
        mpu.rbar.write(0x0000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // SRAM
        mpu.rnr.write(1);
        mpu.rbar.write(0x2000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | NORMAL_SHARED | 1);

        // Peripherals
        mpu.rnr.write(2);
        mpu.rbar.write(0x4000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(3);
        mpu.rbar.write(0x6000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // FSMC
        mpu.rnr.write(4);
        mpu.rbar.write(0xA000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | DEVICE_SHARED | 1);

        // Core peripherals
        mpu.rnr.write(5);
        mpu.rbar.write(0xE000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_512MB | 1);

        // QSPI
        mpu.rnr.write(6);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(27 << 1 | 1 << 28);

        mpu.rnr.write(7);
        mpu.rbar.write(0x9000_0000);
        mpu.rasr.write(FULL_ACCESS | SIZE_8MB | 1 << 17 | 1);

        mpu.ctrl.write(1 | 1 << 2);
    }

    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}

mod test {}
