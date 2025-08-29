//! This *should* demonstrate settime the baud rate used for Modbus communication.
//! It does not, despite doing what is described in the data sheet....

#![no_std]
#![no_main]

use defmt::{info, warn};
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

    let original_baud = RtuBaud::Baud19200;
    let new_baud = RtuBaud::Baud115200;

    let mut motor = Motor::new(&mut uart, original_baud, Duration::from_millis(50));
    info!("Device address: {}", motor.device_address().await);

    motor.set_baud_rate(new_baud.clone()).await.unwrap();

    info!("Power cycle the motor now, plz...");
    Timer::after_secs(1).await;

    uart.set_baudrate(115200);
    let mut motor = Motor::new(&mut uart, new_baud, Duration::from_millis(50));

    loop {
        match motor.device_address().await {
            Ok(addr) => {
                info!("I see the motor again, yey!");
                info!("Device address: {}", addr);
                break;
            }
            Err(e) => {
                warn!("Motor communication failed: {}", e);
                Timer::after_millis(500).await;
            }
        }
    }

    loop {
        info!("Voltage: {} V", motor.voltage().await);

        Timer::after_millis(500).await;
    }
}
