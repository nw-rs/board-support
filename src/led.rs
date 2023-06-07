pub fn init() {
    let rcc = unsafe { &(*crate::pac::RCC::ptr()) };
    let gpiob = unsafe { &(*crate::pac::GPIOB::ptr()) };

    // Enable GPIO B
    rcc.ahb1enr.modify(|_, w| w.gpioben().set_bit());

    // Configure LED pins as outputs
    gpiob
        .moder
        .modify(|_, w| w.moder0().output().moder4().output().moder5().output());
}

pub fn set(red: bool, green: bool, blue: bool) {
    let gpiob = unsafe { &(*crate::pac::GPIOB::ptr()) };
    gpiob
        .odr
        .modify(|_, w| w.odr0().bit(blue).odr4().bit(red).odr5().bit(green));
}

pub fn red() {
    set(true, false, false)
}

pub fn green() {
    set(false, true, false)
}

pub fn blue() {
    set(false, false, true)
}

pub fn off() {
    set(false, false, false)
}
