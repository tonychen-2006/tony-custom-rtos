#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use stm32f4xx_hal::{
    pac,
    prelude::*,
    rcc::Config,
};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(
        Config::DEFAULT.sysclk(48.MHz())
    );

    let gpioa = dp.GPIOA.split(&mut rcc);

    // NUCLEO-F411RE onboard green LED LD2
    let mut led = gpioa.pa5.into_push_pull_output();

    let mut delay = cp.SYST.delay(&rcc.clocks);

    loop {
        led.toggle();
        delay.delay_ms(500);
    }
}