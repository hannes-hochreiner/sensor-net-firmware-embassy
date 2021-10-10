#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// use defmt::panic;
use panic_halt as _;
use embassy::executor::Spawner;
// use embassy_nrf::timer::{Timer};
use core::mem;
// use embassy_nrf::saadc::{Config, OneShot, Sample};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{interrupt, Peripherals};
use embassy::time::{Duration, Timer};
use common::sht4x;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    /* Timer */
    // let mut t = Timer::new_awaitable(p.TIMER0, interrupt::take!(TIMER0));
    // // default frequency is 1MHz
    // t.cc(0).write(1_000);
    /* Timer */
    /* TWIM */
    let mut irq = interrupt::take!(TWIM0_TWIS0_TWI0);
    let mut config = twim::Config::default();
    // let config = twim::Config {
    //     frequency: twim::Frequency::K100,
    //     scl_pullup: true,
    //     sda_pullup: true,
    // };
    config.sda_pullup = true;
    config.scl_pullup = true;
    config.frequency = twim::Frequency::K400;
    let mut twi = Twim::new(&mut p.TWI0, &mut irq, &mut p.P0_22, &mut p.P0_23, config);

    let mut buf = [0u8; 6];
    let _res = twi.write(0x44, &mut [0x89]);
    // t.cc(0).short_compare_clear();
    // t.start();
    // t.cc(0).wait().await;
    // t.stop();
    Timer::after(Duration::from_millis(1)).await;
    let _res = twi.read(0x44, &mut buf);

    let mut sht4 = sht4x::SHT4X {
        i2c: twi,
    };
    
    match sht4.read_serial().await {
        Ok(val) => {
            let tmp = 5;
        },
        Err(e) => {
            let tmp = 5;
        }
    }
    // mem::drop(twi);
    /* TWIM */
    /* SAADC */
    // let config = Config {
    //     resolution: embassy_nrf::saadc::Resolution::_10BIT,
    //     gain: embassy_nrf::saadc::Gain::GAIN1,
    //     oversample: embassy_nrf::saadc::Oversample::OVER16X,
    //     reference: embassy_nrf::saadc::Reference::INTERNAL,
    //     resistor: embassy_nrf::saadc::Resistor::BYPASS,
    //     time: embassy_nrf::saadc::Time::_10US,
    // };
    // // let config = Config::default();
    // let mut saadc = OneShot::new(p.SAADC, interrupt::take!(SAADC), config);

    // loop {
    //     let sample = saadc.sample(&mut p.P0_03).await;
    //     let voltage: f32 = sample as f32 * 0.6 / 1024.0 / 0.4;
    //     let _tmp = 5;
    //     // info!("sample: {=i16}", sample);
    //     // Timer::after(Duration::from_millis(100)).await;
    // }
    /* SAADC */
    
}
