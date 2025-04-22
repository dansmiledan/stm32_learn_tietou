#![no_std]
#![no_main]

use cortex_m_rt::entry;

use panic_halt as _;
use defmt_rtt as _;
use defmt::info;

use stm32f1xx_hal::{
    gpio::{IOPinSpeed, OutputSpeed, PinState::*}, pac::{self}, prelude::*
};


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    info!("Starting up...");
    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split();


    let mut led = gpioc.pc13.into_open_drain_output_with_state(&mut gpioc.crh, High);
    led.set_speed(&mut gpioc.crh, IOPinSpeed::Mhz10);
    // Configure the syst timer to trigger an update every second
    let mut delay = cp.SYST.delay(&clocks);
    info!("init state is high && delay 5s");
    delay.delay_ms(5000_u32);
    loop {
        delay.delay_ms(1000_u32);
        led.set_low();
        delay.delay_ms(1000_u32);
        led.set_high();
    }
}
