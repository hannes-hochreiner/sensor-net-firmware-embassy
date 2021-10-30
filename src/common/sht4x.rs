use embassy::time::{Duration, Timer};
use embassy_traits::i2c::I2c;

pub struct SHT4X<'a, I2C> {
    pub i2c: &'a mut I2C,
    pub address: u8,
}

#[derive(Debug)]
pub enum Sht4xError {
    IdError,
    CrcError,
    Ic2Error,
    TimerError,
}

pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

impl<I2C> SHT4X<'_, I2C> {
    fn crc8(&self, buffer: &[u8]) -> u8 {
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

        self.i2c
            .write(self.address, &com)
            .await
            .map_err(|_| Sht4xError::Ic2Error)?;
        Timer::after(Duration::from_millis(1)).await;
        self.i2c
            .read(self.address, &mut buffer)
            .await
            .map_err(|_| Sht4xError::Ic2Error)?;

        if buffer[2] != self.crc8(&buffer[0..2]) || buffer[5] != self.crc8(&buffer[3..5]) {
            Err(Sht4xError::CrcError)
        } else {
            Ok(u32::from_be_bytes([
                buffer[0], buffer[1], buffer[3], buffer[4],
            ]))
        }
    }

    pub async fn get_measurement(&mut self) -> Result<Measurement, Sht4xError> {
        let mut buffer = [0u8; 6];
        let com = [0xFD];
        self.i2c
            .write(self.address, &com)
            .await
            .map_err(|_| Sht4xError::Ic2Error)?;
        Timer::after(Duration::from_millis(10)).await;
        self.i2c
            .read(self.address, &mut buffer)
            .await
            .map_err(|_| Sht4xError::Ic2Error)?;

        if buffer[2] != self.crc8(&buffer[0..2]) || buffer[5] != self.crc8(&buffer[3..5]) {
            Err(Sht4xError::CrcError)
        } else {
            Ok(Measurement {
                humidity: -6.0
                    + 125.0 * u16::from_be_bytes([buffer[3], buffer[4]]) as f32 / 65535.0,
                temperature: -45.0
                    + 175.0 * u16::from_be_bytes([buffer[0], buffer[1]]) as f32 / 65535.0,
            })
        }
    }
}
