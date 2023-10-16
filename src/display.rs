use crate::pac;

pub const DISPLAY_WIDTH: u16 = 320;
pub const DISPLAY_HEIGHT: u16 = 240;

use mipidsi::models::Model;

pub struct ResetPin;

impl embedded_hal::digital::v2::OutputPin for ResetPin {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let gpioe = unsafe { &*pac::GPIOE::ptr() };

        gpioe.odr.modify(|_, w| w.odr1().low());

        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let gpioe = unsafe { &*pac::GPIOE::ptr() };

        gpioe.odr.modify(|_, w| w.odr1().high());

        Ok(())
    }
}

pub type LcdST7789 = mipidsi::Display<
    crate::fmc_lcd::Lcd<crate::fmc_lcd::SubBank1>,
    mipidsi::models::ST7789,
    ResetPin,
>;

pub fn init_display(ahb_frequency: u32) -> LcdST7789 {
    let rcc = unsafe { &*pac::RCC::ptr() };
    let gpiob = unsafe { &*pac::GPIOB::ptr() };
    let gpioc = unsafe { &*pac::GPIOC::ptr() };
    let gpiod = unsafe { &*pac::GPIOD::ptr() };
    let gpioe = unsafe { &*pac::GPIOE::ptr() };

    rcc.ahb1enr.modify(|_, w| {
        w.gpioben()
            .enabled()
            .gpiocen()
            .enabled()
            .gpioden()
            .enabled()
            .gpioeen()
            .enabled()
            .dma2en()
            .enabled()
    });

    gpiob.moder.modify(|_, w| w.moder11().input());
    gpiob.pupdr.modify(|_, w| w.pupdr11().floating());

    gpioc.moder.modify(|_, w| w.moder8().output());
    gpiod.moder.modify(|_, w| w.moder6().output());
    gpioe
        .moder
        .modify(|_, w| w.moder0().output().moder1().output());

    gpioc.odr.modify(|_, w| w.odr8().high());
    gpiod.odr.modify(|_, w| w.odr6().high());

    let read_timing =
        crate::fmc_lcd::Timing::default().access_mode(crate::fmc_lcd::AccessMode::ModeA);

    let write_timing =
        crate::fmc_lcd::Timing::default().access_mode(crate::fmc_lcd::AccessMode::ModeA);

    let lcd = crate::fmc_lcd::FmcLcd::new(&read_timing, &write_timing);

    let mut delay = cortex_m::delay::Delay::new(
        unsafe { cortex_m::Peripherals::steal().SYST },
        ahb_frequency,
    );

    mipidsi::Builder::with_model(lcd, mipidsi::models::ST7789)
        .with_orientation(mipidsi::Orientation::LandscapeInverted(false))
        .with_display_size(320, 240)
        .init(&mut delay, Some(ResetPin))
        .unwrap()
}

pub fn backlight_on() {
    let gpioe = unsafe { &*pac::GPIOE::ptr() };
    gpioe.odr.modify(|_, w| w.odr0().high());
}

pub fn backlight_off() {
    let gpioe = unsafe { &*pac::GPIOE::ptr() };
    gpioe.odr.modify(|_, w| w.odr0().low());
}
