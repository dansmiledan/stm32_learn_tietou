#![no_std]
#![no_main]

use core::fmt;

use cortex_m_rt::entry;

use panic_halt as _;
use defmt_rtt as _;
use defmt::info;
use fmt::Write;
use stm32f1xx_hal::{
    pac, prelude::*, serial::Serial
};


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    info!("1");

    let mut gpioa = dp.GPIOA.split();

    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .freeze(&mut flash.acr);

    let mut delay = dp.TIM1.delay_ms(&clocks);

    info!("2");
    // define RX/TX pins
    let tx_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx_pin = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
    let mut afio = dp.AFIO.constrain();
    // configure serial
    let (mut tx, _) = Serial::new(dp.USART1, (tx_pin, rx_pin), &mut afio.mapr, 9600.bps(), &clocks).split();


    let mut value: u8 = 0;

    loop {
        info!("start loop {}", value);
        // print some value every 500 ms, value will overflow after 255
        writeln!(tx, "value: {value:02}\r").unwrap();
        info!("value: {:02}", value);
        value = value.wrapping_add(1);
        delay.delay(2.secs());
    }
}
