//! Demonstrates getting all the parameters the motor provides.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config, DataBits, Parity, StopBits},
};
use embassy_time::{Duration, Timer};
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
        Duration::from_millis(50),
    );

    loop {
        info!("=== Parameters:");

        info!("Modbus enable: {}", motor.modbus_enabled().await);
        info!("Drive enable: {}", motor.drive_enabled().await);
        info!("Target speed: {} rpm", motor.target_rpm().await);
        info!("Acceleration: {} rpm/s", motor.acceleration().await);
        info!("Weak magnetic angle: {}", motor.weak_magnetic_angle().await);
        info!("Speed Kp: {}", motor.speed_kp().await);
        info!("Speed I Time: {}", motor.speed_i_time().await);
        info!("Position Kp: {}", motor.position_kp().await);
        info!("Speed feed: {}", motor.speed_feed().await);
        info!("DIR polarity: {}", motor.dir_polarity().await);
        info!(
            "Gear numerator: {}",
            motor.electronic_gear_numerator().await
        );
        info!(
            "Gear denominator: {}",
            motor.electronic_gear_denominator().await
        );
        info!("Target position: {}", motor.target_position().await);
        info!("Alarm: {}", motor.alarm_code().await);
        info!("Current: {} A", motor.current().await);
        info!("Speed: {} rpm", motor.speed().await);
        info!("Voltage: {} V", motor.voltage().await);
        info!("Temperature: {} C", motor.temperature().await);
        info!("PWM: {}", motor.pwm().await);
        info!("Parameter save flag: {}", motor.parameter_save_flag().await);
        info!("Device address: {}", motor.device_address().await);
        info!("Absolute position: {}", motor.absolute_position().await);
        info!(
            "Still maximum allowed current: {}",
            motor.still_maximum_allowed_current().await
        );
        info!("Special function: {}", motor.specific_function().await);

        Timer::after_secs(5).await;
    }
}
