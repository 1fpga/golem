//! Display the pi-top battery status.
//! See http://smartbattery.org/specs/sbdat110.pdf for the Smart Battery Data Specification.
use i2cdev::core::I2CDevice;
use retry::delay::Fixed;
use retry::retry;
use std::ops::Range;
use std::time::Duration;

fn get_reg_(dev: &mut impl I2CDevice, reg: u8, range: Range<i16>) -> Option<i16> {
    // Maximum numbers of tries to read a register before failing.
    const MAX_RETRIES: usize = 20;
    const POLLING_RATE: Duration = Duration::from_nanos(500000);

    retry(Fixed::from(POLLING_RATE).take(MAX_RETRIES), move || {
        dev.smbus_read_word_data(reg)
            .map_err(|_| ())
            .and_then(|value| {
                // Convert the value to a signed 16-bit integer.
                let value = value as i16;
                if range.contains(&value) {
                    Ok(value)
                } else {
                    Err(())
                }
            })
    })
    .ok()
}

pub struct Battery {
    pub capacity: i16,
    pub load_current: i16,
    pub current: i16,
    pub time: i16,
    pub voltage: i16,
}

impl Battery {
    pub fn from_i2cdev(dev: &mut impl I2CDevice) -> Option<Self> {
        let capacity = get_reg_(dev, 0x0D, 0..100)?;
        let load_current = get_reg_(dev, 0x0A, -5000..5000)?;
        let time = if load_current > 0 {
            get_reg_(dev, 0x13, 1..999)?
        } else if load_current < -1 {
            get_reg_(dev, 0x12, 1..960)?
        } else {
            0
        };
        let current = get_reg_(dev, 0x0F, 0..5000)?;
        let voltage = get_reg_(dev, 0x09, 1000..20000)?;

        Some(Self {
            capacity,
            load_current,
            current,
            time,
            voltage,
        })
    }
}
