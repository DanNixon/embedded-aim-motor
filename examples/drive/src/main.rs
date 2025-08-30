//! Demonstrates driving the motor by absolute position in position mode.

#![no_std]
#![no_main]

use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config, DataBits, Parity, StopBits},
};
use embassy_time::{Duration, Instant, Ticker};
use embedded_aim_motor::{Motor, RtuBaud};
use portable_atomic as _;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Hello, world!");

    let mut config = Config::default();
    config.baudrate = 19200;
    config.data_bits = DataBits::DataBits8;
    config.parity = Parity::ParityNone;
    config.stop_bits = StopBits::STOP1;

    const TX_BUFFER_SIZE: usize = 32;
    const RX_BUFFER_SIZE: usize = 32;

    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut uart = BufferedUart::new(p.UART0, p.PIN_0, p.PIN_1, Irqs, tx_buf, rx_buf, config);

    let mut motor = Motor::new(
        &mut uart,
        RtuBaud::Baud19200,
        0x01,
        Duration::from_millis(20),
    );

    motor.set_modbus_enabled(true).await.unwrap();
    motor.set_electronic_gear_numerator(0).await.unwrap();
    motor.set_target_rpm(1000).await.unwrap();
    motor.set_acceleration(3000).await.unwrap();
    motor.set_parameter_save_flag(true).await.unwrap();

    let mut position = 0u32;

    let mut motor_tick = Ticker::every(Duration::from_millis(20));
    let mut report_tick = Ticker::every(Duration::from_millis(100));

    loop {
        match select(motor_tick.next(), report_tick.next()).await {
            Either::First(_) => {
                let start = Instant::now();
                let res = motor.set_absolute_position(position).await;
                let end = Instant::now();

                if res.is_ok() {
                    position = position.saturating_add(5000);
                } else {
                    info!("fuck");
                }

                let delta = end - start;
                debug!("delta = {}ms", delta.as_millis());
            }
            Either::Second(_) => {
                if let Ok(actual) = motor.absolute_position().await {
                    info!(
                        "Position requested/actual: {}/{} (diff {})",
                        position,
                        actual,
                        position - actual
                    );
                }
            }
        }
    }
}
