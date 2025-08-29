use defmt::Format;
use embassy_time::Duration;

#[derive(Debug, Format, Clone)]
pub enum RtuBaud {
    Baud115200,
    Baud38400,
    Baud19200,
    Baud9600,
}

impl RtuBaud {
    // Get the inter-character delay for the baud rate.
    // See https://www.modbus.org/docs/Modbus_over_serial_line_V1_02.pdf
    pub(crate) fn t15(&self) -> Duration {
        match self {
            Self::Baud115200 => Duration::from_micros(750),
            Self::Baud38400 => Duration::from_micros(750),
            Self::Baud19200 => Duration::from_micros(1979),
            Self::Baud9600 => Duration::from_micros(3958),
        }
    }

    // Get the inter-frame delay for the baud rate.
    // See https://www.modbus.org/docs/Modbus_over_serial_line_V1_02.pdf
    pub(crate) fn t35(&self) -> Duration {
        match self {
            Self::Baud115200 => Duration::from_micros(1750),
            Self::Baud38400 => Duration::from_micros(1750),
            Self::Baud19200 => Duration::from_micros(1979),
            Self::Baud9600 => Duration::from_micros(3958),
        }
    }
}

#[derive(Debug, Format, Clone)]
pub enum AlarmCode {
    PowerFailure,
    Overflow,
    Block,
    Overpressure,
}

#[derive(Debug, Format, Clone)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}
