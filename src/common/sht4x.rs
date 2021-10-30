use embassy::time::{Duration, Timer};
use embassy_traits::i2c::I2c;

pub struct SHT4X<'a, I2C> {
    pub i2c: &'a mut I2C,
}

pub enum Sht4xError {
    IdError,
    CrcError,
    TimerError,
}

impl SHT4X<'_, _> {
    fn crc8(buffer: &[u8]) -> u8 {
        let mut rem = 0xff;
        let poly = 0x31;

        for byte in buffer {
            rem ^= byte;

            for _i in 1..9 {
                if rem & 0x80 == 0x80 {
                    rem = (rem << 1) ^ poly;
                } else {
                    rem = rem << 1;
                }
            }
        }

        rem
    }
}

impl<I2C> SHT4X<'_, I2C>
where
    I2C: I2c,
{
    pub async fn read_serial(&mut self) -> Result<u32, Sht4xError> {
        let mut buffer = [0u8; 6];
        let com = [0x89];
        let res = self.i2c.write(0x44, &com).await;
        Timer::after(Duration::from_millis(1)).await;
        let res = self.i2c.read(0x44, &mut buffer).await;

        if buffer[2] != crc8(&buffer[0..2]) || buffer[5] != crc8(&buffer[3..5]) {
            Err(Sht4xError::CrcError)
        } else {
            Ok(u32::from_be_bytes([
                buffer[0], buffer[1], buffer[3], buffer[4],
            ]))
        }
    }
}
