use defmt::Format;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Format, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Transport error")]
    Transport,

    #[error("Communication timeout")]
    Timeout,

    #[error("MODBUS protocol error")]
    Modbus,

    #[error("Response type was not the one expected")]
    UnexpectedResponseType,

    #[error("Expected {0} bytes, but got {1}")]
    UnexpectedResponseLength(usize, usize),

    #[error("Response contains data that was not expected")]
    UnexpectedResponseData,
}
