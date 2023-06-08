#![allow(dead_code)]

use crate::pac;

use embedded_hal::digital::v2::OutputPin;

use core::fmt::Arguments;
use core::fmt::Write;

use heapless::String;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_text::{plugin::tail::Tail, TextBox};

pub const DISPLAY_WIDTH: u16 = 320;
pub const DISPLAY_HEIGHT: u16 = 240;

pub const BG_COLOUR: Rgb565 = Rgb565::BLACK;
pub const TEXT_COLOUR: Rgb565 = Rgb565::GREEN;

const TOP_STRING_SIZE: usize = 1479;
const TOP_LINES: usize = 38;
const BOTTOM_STRING_SIZE: usize = 104;
const BOTTOM_LINES: usize = 3;

macro_rules! output_pin {
    ($struct:ident, $GPIO:ident, $pin:ident) => {
        pub struct $struct;

        impl embedded_hal::digital::v2::OutputPin for $struct {
            type Error = ();

            fn set_low(&mut self) -> Result<(), Self::Error> {
                let gpio = unsafe { &*pac::$GPIO::ptr() };

                gpio.odr.modify(|_, w| w.$pin().low());

                Ok(())
            }
            fn set_high(&mut self) -> Result<(), Self::Error> {
                let gpio = unsafe { &*pac::$GPIO::ptr() };

                gpio.odr.modify(|_, w| w.$pin().high());

                Ok(())
            }
        }
    };
}

output_pin!(ResetPin, GPIOE, odr1);
output_pin!(PowerPin, GPIOC, odr8);
output_pin!(ExtdCmdPin, GPIOD, odr6);
output_pin!(BacklightPin, GPIOE, odr0);

pub type Color = Rgb565;

pub type LcdST7789 = mipidsi::Display<
    crate::fmc_lcd::Lcd<crate::fmc_lcd::SubBank1>,
    mipidsi::models::ST7789,
    ResetPin,
>;

pub struct Display {
    pub display: LcdST7789,
    pub top: String<TOP_STRING_SIZE>,
    pub bottom: String<BOTTOM_STRING_SIZE>,
    backlight_state: u8,
}

impl Display {
    pub fn new(ahb_frequency: u32) -> Self {
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

        let ns_to_cycles = |ns: u32| (ahb_frequency / 1_000_000) * ns / 1000;

        let tedge: u32 = 15;
        let twc: u32 = 66;
        let trcfm: u32 = 450;
        let twrl: u32 = 15;
        let trdlfm: u32 = 355;

        let trdatast = trdlfm + tedge;

        let read_data_cycles = ns_to_cycles(trdatast);

        let read_addrsetup_cycles = ns_to_cycles(trcfm - trdatast);

        let read_timing = crate::fmc_lcd::Timing::default()
            .data(read_data_cycles as u8)
            .address_setup(read_addrsetup_cycles as u8)
            .address_hold(0)
            .bus_turnaround(0)
            .access_mode(crate::fmc_lcd::AccessMode::ModeA);

        let twdatast = twrl + tedge;

        let write_data_cycles = ns_to_cycles(twdatast);

        let write_addrsetup_cycles = ns_to_cycles(twc - twdatast) - 1;

        let write_timing = crate::fmc_lcd::Timing::default()
            .data(write_data_cycles as u8)
            .address_setup(write_addrsetup_cycles as u8)
            .address_hold(0)
            .bus_turnaround(0)
            .access_mode(crate::fmc_lcd::AccessMode::ModeA);

        let _ = PowerPin.set_high();

        let _ = BacklightPin.set_high();

        let _ = ExtdCmdPin.set_high();

        let lcd = crate::fmc_lcd::FmcLcd::new(&read_timing, &write_timing);

        let mut delay = cortex_m::delay::Delay::new(
            unsafe { cortex_m::Peripherals::steal().SYST },
            ahb_frequency,
        );

        let _ = ResetPin.set_low();
        delay.delay_us(5000u32);
        let _ = ResetPin.set_high();
        delay.delay_us(10000u32);
        let _ = ResetPin.set_low();
        delay.delay_us(20000u32);
        let _ = ResetPin.set_high();
        delay.delay_us(10000u32);

        let mut display = mipidsi::Builder::with_model(lcd, mipidsi::models::ST7789)
            .with_orientation(mipidsi::Orientation::LandscapeInverted(false))
            .init(&mut delay, Some(ResetPin))
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        Self {
            display,
            top: String::new(),
            bottom: String::new(),
            backlight_state: 1,
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.display.clear(color).unwrap();
    }

    pub fn set_backlight(&mut self, target: u8) {
        if target == 0 {
            let _ = BacklightPin.set_low();
        } else {
            let _ = BacklightPin.set_high();
        }
        self.backlight_state = target;
    }

    pub fn write_bottom_to_top(mut self) -> Self {
        let bottom_content = self.bottom;
        self.bottom = String::new();
        self.write_top_fmt(format_args!("\n{}", &bottom_content));
        self
    }

    pub fn write_top(&mut self, text: &str) {
        if text.len() > TOP_STRING_SIZE {
            self.top.clear();
            self.top
                .push_str(unsafe {
                    text.get_unchecked((text.len() - TOP_STRING_SIZE)..(text.len() - 1))
                })
                .unwrap();
        } else {
            if self.top.len() + text.len() > TOP_STRING_SIZE {
                let old_top = self.top.clone();
                self.top.clear();
                self.top
                    .push_str(unsafe {
                        let t = &old_top.as_str().get_unchecked(
                            (old_top.len() + text.len() - TOP_STRING_SIZE)..(old_top.len() - 1),
                        );
                        t
                    })
                    .unwrap();
            }
            self.top.push_str(text).unwrap();
        }
    }

    pub fn write_top_fmt(&mut self, args: Arguments<'_>) {
        self.write_fmt(args).unwrap()
    }

    pub fn write_bottom(&mut self, text: &str, redraw: bool) -> bool {
        if !(self.bottom.len() + text.len() > BOTTOM_STRING_SIZE) {
            self.bottom.write_str(text).unwrap();
            if redraw {
                self.draw_bottom(true);
            }
            true
        } else {
            false
        }
    }

    pub fn clear_bottom(&mut self, redraw: bool) {
        self.bottom.clear();
        if redraw {
            self.draw_bottom(true);
        }
    }

    pub fn pop_bottom(&mut self, redraw: bool) {
        self.bottom.pop();
        if redraw {
            self.draw_bottom(true);
        }
    }

    pub fn draw_bottom(&mut self, clear: bool) {
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);

        let bottom_bounds = Rectangle::new(
            Point::new(3, DISPLAY_HEIGHT as i32 - 16),
            self.display.size() - Size::new(6, 0),
        );

        if clear {
            bottom_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }

        TextBox::new(&self.bottom, bottom_bounds, character_style)
            .add_plugin(Tail)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn draw_top(&mut self, clear: bool) {
        let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN);

        let top_bounds = Rectangle::new(Point::new(3, 5), self.display.size() - Size::new(6, 15));

        if clear {
            top_bounds
                .into_styled(PrimitiveStyleBuilder::new().fill_color(BG_COLOUR).build())
                .draw(&mut self.display)
                .unwrap();
        }

        TextBox::new(&self.top, top_bounds, character_style)
            .add_plugin(Tail)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn draw_all(&mut self) {
        self.clear(BG_COLOUR);
        self.draw_bottom(false);
        self.draw_top(false);
    }
}

impl Write for Display {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_top(s);
        Ok(())
    }
}
