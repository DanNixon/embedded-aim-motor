#![no_std]

mod error;
mod motor;
mod types;

pub use error::{Error, Result};
pub use motor::Motor;
pub use types::{AlarmCode, Direction, RtuBaud};
