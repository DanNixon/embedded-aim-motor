use super::Motor;
use crate::{AlarmCode, Direction, Error, Result};
use embassy_time::Duration;

impl<I: embedded_io_async::Read + embedded_io_async::Write> Motor<I> {
    pub async fn modbus_enabled(&mut self) -> Result<bool> {
        self.read_one_word_parameter(0x00, |v| match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::UnexpectedResponseData),
        })
        .await
    }

    pub async fn set_modbus_enabled(&mut self, value: bool) -> Result<()> {
        self.write_one_word_parameter(0x00, value, |v| match v {
            false => Ok(0),
            true => Ok(1),
        })
        .await
    }

    pub async fn drive_enabled(&mut self) -> Result<bool> {
        self.read_one_word_parameter(0x01, |v| match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::UnexpectedResponseData),
        })
        .await
    }

    pub async fn set_drive_enabled(&mut self, value: bool) -> Result<()> {
        self.write_one_word_parameter(0x01, value, |v| match v {
            false => Ok(0),
            true => Ok(1),
        })
        .await
    }

    /// Gets the target speed in RPM
    ///
    /// In speed mode, this is the target speed.
    /// In position mode, this is the maximum speed.
    pub async fn target_rpm(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x02, Ok).await
    }

    /// Sets the target speed in RPM
    ///
    /// 0-3000 RPM.
    /// In speed mode, this is the target speed.
    /// In position mode, this is the maximum speed.
    pub async fn set_target_rpm(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x02, value, Ok).await
    }

    pub async fn acceleration(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x03, Ok).await
    }

    pub async fn set_acceleration(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x03, value, Ok).await
    }

    pub async fn weak_magnetic_angle(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x04, Ok).await
    }

    pub async fn set_weak_magnetic_angle(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x04, value, Ok).await
    }

    pub async fn speed_kp(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x05, Ok).await
    }

    pub async fn set_speed_kp(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x05, value, Ok).await
    }

    pub async fn speed_i_time(&mut self) -> Result<Duration> {
        self.read_one_word_parameter(0x06, |v| Ok(Duration::from_millis(v.into())))
            .await
    }

    pub async fn set_speed_i_time(&mut self, value: Duration) -> Result<()> {
        self.write_one_word_parameter(0x06, value, |v| Ok(v.as_millis() as u16))
            .await
    }

    pub async fn position_kp(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x07, Ok).await
    }

    pub async fn set_position_kp(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x07, value, Ok).await
    }

    pub async fn speed_feed(&mut self) -> Result<f32> {
        self.read_one_word_parameter(0x08, |v| Ok(v as f32 / 327.))
            .await
    }

    pub async fn set_speed_feed(&mut self, value: f32) -> Result<()> {
        self.write_one_word_parameter(0x08, value, |v| Ok((v * 327.) as u16))
            .await
    }

    pub async fn dir_polarity(&mut self) -> Result<Direction> {
        self.read_one_word_parameter(0x09, |v| match v {
            0 => Ok(Direction::CounterClockwise),
            1 => Ok(Direction::Clockwise),
            _ => Err(Error::UnexpectedResponseData),
        })
        .await
    }

    pub async fn set_dir_polarity(&mut self, value: Direction) -> Result<()> {
        self.write_one_word_parameter(0x09, value, |v| match v {
            Direction::CounterClockwise => Ok(0),
            Direction::Clockwise => Ok(1),
        })
        .await
    }

    pub async fn electronic_gear_numerator(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x0A, Ok).await
    }

    /// Set numerator of electronic gear ratio
    ///
    /// 0-65535
    /// 0 enables special functions
    pub async fn set_electronic_gear_numerator(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x0A, value, Ok).await
    }

    pub async fn electronic_gear_denominator(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x0B, Ok).await
    }

    /// Set denominator of electronic gear ratio
    ///
    /// 1-65535
    pub async fn set_electronic_gear_denominator(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x0B, value, Ok).await
    }

    pub async fn target_position(&mut self) -> Result<u32> {
        self.read_two_word_parameter(0x0C, |msb, lsb| {
            let msb = msb.to_be_bytes();
            let lsb = lsb.to_be_bytes();
            Ok(u32::from_be_bytes([lsb[0], lsb[1], msb[0], msb[1]]))
        })
        .await
    }

    pub async fn set_target_position(&mut self, value: u32) -> Result<()> {
        self.write_two_word_parameter(0x0C, value, |v| {
            let data = v.to_be_bytes();
            let msb = u16::from_be_bytes([data[0], data[1]]);
            let lsb = u16::from_be_bytes([data[2], data[3]]);
            Ok([lsb, msb])
        })
        .await
    }

    pub async fn alarm_code(&mut self) -> Result<Option<AlarmCode>> {
        self.read_one_word_parameter(0x0E, |v| match v {
            0 => Ok(None),
            0x10 => Ok(Some(AlarmCode::PowerFailure)),
            0x12 => Ok(Some(AlarmCode::Overflow)),
            0x14 => Ok(Some(AlarmCode::Block)),
            0x15 => Ok(Some(AlarmCode::Overpressure)),
            _ => Err(Error::UnexpectedResponseData),
        })
        .await
    }

    pub async fn current(&mut self) -> Result<f32> {
        self.read_one_word_parameter(0x0F, |v| Ok(v as f32 / 2000.))
            .await
    }

    pub async fn speed(&mut self) -> Result<f32> {
        self.read_one_word_parameter(0x10, |v| Ok(v as f32 / 10.))
            .await
    }

    pub async fn voltage(&mut self) -> Result<f32> {
        self.read_one_word_parameter(0x11, |v| Ok(v as f32 / 327.))
            .await
    }

    pub async fn temperature(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x12, Ok).await
    }

    pub async fn pwm(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x13, Ok).await
    }

    pub async fn parameter_save_flag(&mut self) -> Result<bool> {
        self.read_one_word_parameter(0x14, |v| match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::UnexpectedResponseData),
        })
        .await
    }

    pub async fn set_parameter_save_flag(&mut self, value: bool) -> Result<()> {
        self.write_one_word_parameter(0x14, value, |v| match v {
            false => Ok(0),
            true => Ok(1),
        })
        .await
    }

    pub async fn device_address(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x15, Ok).await
    }

    pub async fn absolute_position(&mut self) -> Result<u32> {
        self.read_two_word_parameter(0x16, |msb, lsb| {
            let msb = msb.to_be_bytes();
            let lsb = lsb.to_be_bytes();
            Ok(u32::from_be_bytes([lsb[0], lsb[1], msb[0], msb[1]]))
        })
        .await
    }

    pub async fn set_absolute_position(&mut self, value: u32) -> Result<()> {
        self.write_two_word_parameter(0x16, value, |v| {
            let data = v.to_be_bytes();
            let msb = u16::from_be_bytes([data[0], data[1]]);
            let lsb = u16::from_be_bytes([data[2], data[3]]);
            Ok([lsb, msb])
        })
        .await
    }

    pub async fn still_maximum_allowed_current(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x18, Ok).await
    }

    pub async fn set_still_maximum_allowed_current(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x18, value, Ok).await
    }

    pub async fn specific_function(&mut self) -> Result<u16> {
        self.read_one_word_parameter(0x19, Ok).await
    }

    pub async fn set_specific_function(&mut self, value: u16) -> Result<()> {
        self.write_one_word_parameter(0x19, value, Ok).await
    }
}
