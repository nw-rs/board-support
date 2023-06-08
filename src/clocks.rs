pub const HCLK: u32 = 216_000_000;
pub const HSI: u32 = 16_000_000;

pub fn init_clocks() {
    let rcc = unsafe { &(*crate::pac::RCC::ptr()) };
    let pwr = unsafe { &(*crate::pac::PWR::ptr()) };
    let flash = unsafe { &(*crate::pac::FLASH::ptr()) };

    // Turn on HSI
    rcc.cr.modify(|_, w| w.hsion().on());
    while rcc.cr.read().hsirdy().is_not_ready() {}

    // Switch to HSI
    rcc.cfgr.modify(|_, w| w.sw().hsi());
    while !rcc.cfgr.read().sws().is_hsi() {}

    // Enable HSE using crystal oscillator
    rcc.cr.modify(|_, w| w.hsebyp().not_bypassed());
    rcc.cr.modify(|_, w| w.hseon().on());
    while rcc.cr.read().hserdy().is_not_ready() {}

    rcc.cr.modify(|_, w| w.pllon().off());

    rcc.sscgr.write(|w| {
        w.modper()
            .bits(250)
            .incstep()
            .bits(25)
            .spreadsel()
            .center()
            .sscgen()
            .set_bit()
    });

    // Configure PLL and set its source to the HSE
    rcc.pllcfgr.modify(|_, w| unsafe {
        w.pllm().bits(4);
        w.plln().bits(216);
        w.pllp().bits(2);
        w.pllq().bits(9);
        w.pllsrc().hse()
    });

    // Enable PWR domain and set correct voltage scaling
    rcc.apb1enr
        .modify(|_, w| w.pwren().enabled().rtcapben().enabled());
    pwr.cr1.modify(|_, w| w.vos().scale1());

    // Enable PLL
    rcc.cr.modify(|_, w| w.pllon().on());
    while rcc.cr.read().pllrdy().is_not_ready() {}

    // Enable power over-drive mode
    pwr.cr1.modify(|_, w| w.oden().set_bit());
    while !pwr.csr1.read().odrdy().bit_is_set() {}

    // Switch the voltage regulator from normal mode to over-drive mode
    pwr.cr1.modify(|_, w| w.odswen().set_bit());
    while !pwr.csr1.read().odswrdy().bit_is_set() {}

    // Enable PLL48CLK
    rcc.dckcfgr2.modify(|_, w| w.ck48msel().pll());

    flash
        .acr
        .write(|w| w.latency().ws7().prften().enabled().arten().set_bit());

    rcc.apb2enr.write(|w| {
        w.adc1en()
            .set_bit()
            .syscfgen()
            .set_bit()
            .usart6en()
            .set_bit()
    });

    rcc.cfgr
        .modify(|_, w| w.ppre1().div4().ppre2().div2().hpre().div1());

    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}

    cortex_m::asm::delay(16);
}
