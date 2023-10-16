use stm32h7::stm32h735 as pac;

pub fn setup_fmc_pins() {
    let peripherals: pac::Peripherals = unsafe { pac::Peripherals::steal() };
    let gpioa = peripherals.GPIOA;
    let gpiob = peripherals.GPIOB;
    let gpioc = peripherals.GPIOC;
    let gpiod = peripherals.GPIOD;
    let gpioe = peripherals.GPIOE;

    // TODO: pin D11 needs to be checked (FMC address mode or something) requires hardware probing with multimeter or a schematic...

    gpioa
        .otyper
        .modify(|_, w| w.ot4().push_pull().ot5().push_pull());
    gpiob
        .otyper
        .modify(|_, w| w.ot14().push_pull().ot15().push_pull());
    gpioc
        .otyper
        .modify(|_, w| w.ot0().push_pull().ot7().push_pull().ot12().push_pull());
    gpiod.otyper.modify(|_, w| {
        w.ot0()
            .push_pull()
            .ot1()
            .push_pull()
            .ot2()
            .push_pull()
            .ot4()
            .push_pull()
            .ot5()
            .push_pull()
            .ot8()
            .push_pull()
            .ot9()
            .push_pull()
            .ot10()
            .push_pull()
            .ot11()
            .push_pull()
            .ot14()
            .push_pull()
            .ot15()
            .push_pull()
    });
    gpioe
        .otyper
        .modify(|_, w| w.ot7().push_pull().ot8().push_pull());

    gpioa
        .moder
        .modify(|_, w| w.moder4().alternate().moder5().alternate());
    gpiob
        .moder
        .modify(|_, w| w.moder14().alternate().moder15().alternate());
    gpioc
        .moder
        .modify(|_, w| w.moder0().alternate().moder7().alternate().moder12().alternate());
    gpiod.moder.modify(|_, w| {
        w.moder0()
            .alternate()
            .moder1()
            .alternate()
            .moder2()
            .alternate()
            .moder4()
            .alternate()
            .moder5()
            .alternate()
            .moder8()
            .alternate()
            .moder9()
            .alternate()
            .moder10()
            .alternate()
            .moder11()
            .alternate()
            .moder14()
            .alternate()
            .moder15()
            .alternate()
    });
    gpioe
        .moder
        .modify(|_, w| w.moder7().alternate().moder8().alternate());

    gpioa
        .ospeedr
        .modify(|_, w| w.ospeedr4().very_high_speed().ospeedr5().very_high_speed());
    gpiob.ospeedr.modify(|_, w| {
        w.ospeedr14()
            .very_high_speed()
            .ospeedr15()
            .very_high_speed()
    });
    gpioc.ospeedr.modify(|_, w| {
        w.ospeedr0()
            .very_high_speed()
            .ospeedr7()
            .very_high_speed()
            .ospeedr12()
            .very_high_speed()
    });
    gpiod.ospeedr.modify(|_, w| {
        w.ospeedr0()
            .very_high_speed()
            .ospeedr1()
            .very_high_speed()
            .ospeedr2()
            .very_high_speed()
            .ospeedr4()
            .very_high_speed()
            .ospeedr5()
            .very_high_speed()
            .ospeedr8()
            .very_high_speed()
            .ospeedr9()
            .very_high_speed()
            .ospeedr10()
            .very_high_speed()
            .ospeedr11()
            .very_high_speed()
            .ospeedr14()
            .very_high_speed()
            .ospeedr15()
            .very_high_speed()
    });
    gpioe
        .ospeedr
        .modify(|_, w| w.ospeedr7().very_high_speed().ospeedr8().very_high_speed());

    gpioa.afrl.modify(|_, w| w.afr4().af12().afr5().af12());
    gpiob.afrh.modify(|_, w| w.afr14().af12().afr15().af12());
    gpioc.afrl.modify(|_, w| w.afr0().af1().afr7().af9());
    gpioc.afrh.modify(|_, w| w.afr12().af1());
    gpiod.afrl.modify(|_, w| {
        w.afr0()
            .af12()
            .afr1()
            .af12()
            .afr2()
            .af1()
            .afr4()
            .af12()
            .afr5()
            .af12()
    });
    gpiod.afrh.modify(|_, w| {
        w.afr8()
            .af12()
            .afr9()
            .af12()
            .afr11()
            .af12()
            .afr14()
            .af12()
            .afr15()
            .af12()
    });
    gpioe.afrl.modify(|_, w| w.afr7().af12());
    gpioe.afrh.modify(|_, w| w.afr8().af12());
}

pub fn setup_qspi_pins() {
    let peripherals: pac::Peripherals = unsafe { pac::Peripherals::steal() };
    let gpioa = peripherals.GPIOA;
    let gpiob = peripherals.GPIOB;
    let gpioc = peripherals.GPIOC;
    let gpioe = peripherals.GPIOE;

    // TODO: check actual QSPI pin hookup

    gpioa
        .otyper
        .modify(|_, w| w.ot1().push_pull().ot3().push_pull());
    gpiob
        .otyper
        .modify(|_, w| w.ot0().push_pull().ot10().push_pull());
    gpioc.otyper.modify(|_, w| w.ot3().push_pull());
    gpioe.otyper.modify(|_, w| w.ot2().push_pull());

    gpioa
        .moder
        .modify(|_, w| w.moder1().alternate().moder3().alternate());
    gpiob
        .moder
        .modify(|_, w| w.moder0().alternate().moder10().alternate());
    gpioc.moder.modify(|_, w| w.moder3().alternate());
    gpioe.moder.modify(|_, w| w.moder2().alternate());

    gpioa.ospeedr.modify(|_,w|w.ospeedr1().high_speed().ospeedr3().high_speed());
    gpiob
        .ospeedr
        .modify(|_, w| w.ospeedr0().very_high_speed().ospeedr10().very_high_speed());
    gpioc.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());
    gpioe.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

    gpioa.afrl.modify(|_,w|w.afr1().af9().afr3().af12());
    gpiob.afrl.modify(|_,w|w.afr0().af4());
    gpiob.afrh.modify(|_,w|w.afr10().af9());
    gpioc.afrl.modify(|_,w|w.afr3().af9());
    gpioe.afrl.modify(|_,w|w.afr2().af9());
}
