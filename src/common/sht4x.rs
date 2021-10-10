use embassy_traits::i2c::{I2c};
// use embedded_hal::blocking::i2c::Write;
use embassy::time::{Duration, Timer};

pub struct SHT4X<I2C> {
    pub i2c: I2C
}

pub enum Sht4xError {
    IdError,
    CrcError,
    TimerError,
}

impl<I2C> SHT4X<I2C>
where
    I2C: I2c,
{
    pub async fn read_serial(&mut self) -> Result<u32, Sht4xError> {
        let mut buffer = [0u8; 6];
        let com = [0x89];
        let res = self.i2c.write(0x44, &com).await;
        Timer::after(Duration::from_millis(1)).await;
        let res = self.i2c.read(0x44, &mut buffer).await;

        Ok(u32::from_be_bytes([
            buffer[0], buffer[1], buffer[3], buffer[4],
        ]))
    }
}
