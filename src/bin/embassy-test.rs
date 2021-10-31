#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// use defmt::panic;
// use embassy::executor::Spawner;
use panic_halt as _;
// use embassy_nrf::timer::{Timer};
use core::mem;
// use embassy_nrf::saadc::{Config, OneShot, Sample};
use common::sht4x;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::radio::{self, Radio};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{
    config::{Config, HfclkSource, LfclkSource},
    interrupt, Peripherals,
};

static EXECUTOR: Forever<embassy::executor::Executor> = Forever::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let executor = EXECUTOR.put(embassy::executor::Executor::new());
    let mut embassy_config = Config::default();

    embassy_config.hfclk_source = HfclkSource::ExternalXtal;
    embassy_config.lfclk_source = LfclkSource::ExternalXtal;

    let p = embassy_nrf::init(embassy_config);

    executor.run(|spawner| {
        spawner.spawn(main(p)).unwrap();
    })
}

#[embassy::task]
async fn main(mut p: Peripherals) {
    /* IRQs */
    let mut twim_irq = interrupt::take!(TWIM0_TWIS0_TWI0);
    let mut radio_irq = interrupt::take!(RADIO);
    /* IRQs */
    /* TWIM */
    let mut twi = Twim::new(
        &mut p.TWI0,
        &mut twim_irq,
        &mut p.P0_22,
        &mut p.P0_23,
        get_twim_config(),
    );
    let mut sht4 = sht4x::SHT4X {
        i2c: &mut twi,
        address: 0x44,
    };

    let serial = sht4.read_serial().await.unwrap();

    mem::drop(sht4);
    mem::drop(twi);
    /* TWIM */

    let mut index = 0u32;

    loop {
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
        /* Measurement */
        let mut twi = Twim::new(
            &mut p.TWI0,
            &mut twim_irq,
            &mut p.P0_22,
            &mut p.P0_23,
            get_twim_config(),
        );
        let mut sht4 = sht4x::SHT4X {
            i2c: &mut twi,
            address: 0x44,
        };

        let measurement = sht4.get_measurement(&sht4x::Precision::High).await.unwrap();

        mem::drop(sht4);
        mem::drop(twi);
        /* Measurement */
        /* Transmission */
        let mut radio = Radio::new(&mut p.RADIO, &mut radio_irq, get_radio_config());

        // create package
        let mut packet: [u8; 29] = [0; 29];
        let device_id = 0u64;
        let part_id = 0u32;

        packet[0] = 29;
        packet[1..3].copy_from_slice(&4u16.to_le_bytes()[..]);
        packet[3..11].copy_from_slice(&device_id.to_le_bytes()[..]);
        packet[11..15].copy_from_slice(&part_id.to_le_bytes()[..]);
        packet[15..19].copy_from_slice(&index.to_le_bytes()[..]);
        packet[19..21].copy_from_slice(&serial.to_le_bytes()[0..2]);
        packet[21..25].copy_from_slice(&measurement.temperature.to_le_bytes()[..]);
        packet[25..29].copy_from_slice(&measurement.humidity.to_le_bytes()[..]);

        radio
            .transmit(&get_radio_tx_config(), &packet)
            .await
            .unwrap();
        /* Transmission */

        index += 1;

        /* Reception */
        // let mut rx_packet: [u8; 256] = [0; 256];
        // let mut radio_rx_config = radio::RxConfig::default();

        // radio_rx_config.rx_address_0_active = true;

        // let _rssi = radio
        //     .receive(&radio_rx_config, &mut rx_packet)
        //     .await
        //     .unwrap();

        // let _packet_length = rx_packet[0];
        // let _packet_type = u16::from_le_bytes([rx_packet[1], rx_packet[2]]);

        // mem::drop(radio);
        /* Reception */

        Timer::after(Duration::from_millis(1000)).await;
    }
}

fn get_radio_config() -> radio::Config {
    let mut radio_config = radio::Config::default();

    radio_config.mode = radio::Mode::Ble1Mbit;
    radio_config.frequency = radio::Frequency::new(2490).unwrap();
    radio_config.length_length = radio::Value4Bit::new(8).unwrap();
    radio_config.base_address_0 = 0xABCDABCD;
    radio_config.prefix_0 = 0xEF;
    radio_config.endianness = radio::Endianness::Big;
    radio_config.crc_length = radio::CrcLength::Crc3bytes;
    radio_config.crc_poly = radio::Value24Bit::new(0b00000000_00000110_01011011).unwrap();

    radio_config
}

fn get_radio_tx_config() -> radio::TxConfig {
    let mut radio_tx_config = radio::TxConfig::default();

    radio_tx_config.tx_power = radio::TxPower::Pos4dBm;

    radio_tx_config
}

fn get_twim_config() -> twim::Config {
    let mut config = twim::Config::default();

    config.sda_pullup = true;
    config.scl_pullup = true;
    config.frequency = twim::Frequency::K400;

    config
}
