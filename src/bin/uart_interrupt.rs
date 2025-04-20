#![no_std]
#![no_main]
use cortex_m_rt::entry;
// pick a panicking behavior
use panic_halt as _;
use defmt_rtt as _; // 关键！隐式初始化 RTT 通道
// use rtt_target::rtt_init_defmt;
use defmt::{warn};
// use panic_rtt_target as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                           // use panic_abort as _; // requires nightly
                           // use panic_itm as _; // logs messages over ITM; requires ITM support
                           // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use stm32f1xx_hal::{
    pac::{self, interrupt, USART1},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};
// mod logger;

static mut RX: Option<Rx<USART1>> = None;
static mut TX: Option<Tx<USART1>> = None;

#[entry]
fn main() -> ! {
    // rtt_init_defmt!();
    // rtt_init_print!();
    // logger::init(); warn!("Starting up...");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    warn!("Starting up...");
    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split();
    let mut gpioa = dp.GPIOA.split();

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = Serial::new(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    let (mut serial_tx, mut serial_rx) = serial.split();
    // serial_tx.listen();
    serial_rx.listen();
    // serial_rx.listen_idle();

    cortex_m::interrupt::free( |_| unsafe {
        // use static mut must in unsafe block
        RX.replace(serial_rx);
        TX.replace(serial_tx);
    });

    warn!("before unmask");
    unsafe {
        // cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
        // cortex_m::peripheral::NVIC::setPriority(pac::Interrupt::USART1, 1);
        // cortex_m::peripheral::NVIC::set_priority(&mut self, interrupt, prio);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
    }
    warn!("after unmask");

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut delay = cp.SYST.delay(&clocks);
    loop {
        delay.delay_ms(1000_u32);
        led.set_high();
        delay.delay_ms(1000_u32);
        led.set_low();
    }
}

const BUFFER_LEN: usize = 8;
static mut BUFFER: &mut [u8; BUFFER_LEN] = &mut [0; BUFFER_LEN];
static mut WIDX: usize = 0;

unsafe fn write(buf: &[u8]) {
    if let Some(tx) = &mut TX {
        buf.iter()
            .for_each(|w| if let Err(_err) = nb::block!(tx.write(*w)) {})
    }
}

#[interrupt]
unsafe fn USART1() {
    cortex_m::interrupt::free(|_| {
        if let Some(rx) = &mut RX {
            if rx.is_rx_not_empty() {
                warn!("enter int");
                if let Ok(w) = nb::block!(rx.read()) {
                    warn!("writing {}", w);
                    BUFFER[WIDX] = w;
                    WIDX += 1;
                    if WIDX >= BUFFER_LEN - 1 {
                        write(&BUFFER[..]);
                        WIDX = 0;
                    }
                }
                rx.listen_idle();
                warn!("exit int");
            } else if rx.is_idle() {
                warn!("en int");
                rx.unlisten_idle();
                write(&BUFFER[0..WIDX]);
                WIDX = 0;
                warn!("e int");
            }
        }
    });
}