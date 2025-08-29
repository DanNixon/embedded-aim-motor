mod parameters;

use crate::{Error, Result, RtuBaud};
use defmt::debug;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use modbus_core::{
    Data, FunctionCode, Request, RequestPdu, Response,
    rtu::{Header, RequestAdu},
};

pub struct Motor<I: embedded_io_async::Read + embedded_io_async::Write> {
    comm: I,

    t15: Duration,
    t35: Duration,
    response_timeout: Duration,

    address: u8,

    buffer: [u8; 64],
    earliest_next_frame: Instant,
}

impl<I: embedded_io_async::Read + embedded_io_async::Write> Motor<I> {
    pub fn new(comm: I, baud: RtuBaud, response_timeout: Duration) -> Self {
        Self {
            comm,
            t15: baud.t15(),
            t35: baud.t35(),
            response_timeout,
            address: 0x01,
            buffer: [0u8; 64],
            earliest_next_frame: Instant::now(),
        }
    }

    async fn modbus_transaction<'a>(&'a mut self, req: RequestPdu<'a>) -> Result<Response<'a>> {
        // Ensure we wait for at least the inter-frame delay
        Timer::at(self.earliest_next_frame).await;

        // Create request
        let request = RequestAdu {
            hdr: Header {
                slave: self.address,
            },
            pdu: req,
        };

        // Encode request
        let n = modbus_core::rtu::client::encode_request(request, &mut self.buffer)
            .map_err(|_| Error::Transport)?;
        let data = &self.buffer[..n];
        debug!("Encoded request: ({}) {:x}", n, data);

        // Send request
        self.comm
            .write_all(data)
            .await
            .map_err(|_| Error::Transport)?;

        let mut timeout = self.response_timeout;
        let mut total_read = 0;

        // Receive data
        'rx: loop {
            match with_timeout(timeout, self.comm.read(&mut self.buffer[total_read..])).await {
                Ok(Ok(n)) => {
                    total_read += n;
                    self.earliest_next_frame = Instant::now() + self.t35;
                }
                Ok(Err(_)) => {
                    return Err(Error::Transport);
                }
                Err(_) => break 'rx,
            }

            timeout = self.t15;
        }

        if total_read == 0 {
            // Timeout if nothing has been received
            Err(Error::Timeout)
        } else {
            let data = &self.buffer[..total_read];
            debug!("Received: ({}) {:x}", total_read, data);

            // Try to parse the response
            let response = modbus_core::rtu::client::decode_response(data)
                .map_err(|_| Error::Transport)?
                .ok_or(Error::Modbus)?;

            Ok(response.pdu.0.map_err(|_| Error::Modbus)?)
        }
    }

    async fn read_one_word_parameter<T, F>(&mut self, address: u16, transform: F) -> Result<T>
    where
        F: Fn(u16) -> Result<T>,
    {
        let request = RequestPdu(Request::ReadHoldingRegisters(address, 1));

        match self.modbus_transaction(request).await? {
            Response::ReadHoldingRegisters(data) => {
                if data.len() == 1 {
                    let raw = data.get(0).unwrap();
                    Ok(transform(raw)?)
                } else {
                    Err(Error::UnexpectedResponseLength(data.len(), 1))
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }

    async fn read_two_word_parameter<T, F>(&mut self, address: u16, transform: F) -> Result<T>
    where
        F: Fn(u16, u16) -> Result<T>,
    {
        let request = RequestPdu(Request::ReadHoldingRegisters(address, 2));

        match self.modbus_transaction(request).await? {
            Response::ReadHoldingRegisters(data) => {
                if data.len() == 2 {
                    let raw_0 = data.get(0).unwrap();
                    let raw_1 = data.get(1).unwrap();
                    Ok(transform(raw_0, raw_1)?)
                } else {
                    Err(Error::UnexpectedResponseLength(data.len(), 2))
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }

    async fn write_one_word_parameter<T, F>(
        &mut self,
        address: u16,
        value: T,
        transform: F,
    ) -> Result<()>
    where
        F: Fn(T) -> Result<u16>,
    {
        let data = transform(value)?;

        let request = RequestPdu(Request::WriteSingleRegister(address, data));

        match self.modbus_transaction(request).await? {
            Response::WriteSingleRegister(a, d) => {
                if a == address && d == data {
                    Ok(())
                } else {
                    Err(Error::UnexpectedResponseData)
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }

    async fn write_two_word_parameter<T, F>(
        &mut self,
        address: u16,
        value: T,
        transform: F,
    ) -> Result<()>
    where
        F: Fn(T) -> Result<[u16; 2]>,
    {
        let data = transform(value)?;

        let mut buff = [0u8; 4];
        let data = Data::from_words(&data, &mut buff).unwrap();

        let request = RequestPdu(Request::WriteMultipleRegisters(address, data));

        match self.modbus_transaction(request).await? {
            Response::WriteMultipleRegisters(a, 2) => {
                if a == address {
                    Ok(())
                } else {
                    Err(Error::UnexpectedResponseData)
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }

    pub async fn set_baud_rate(&mut self, baud: RtuBaud) -> Result<()> {
        let baud = match baud {
            RtuBaud::Baud115200 => 803,
            RtuBaud::Baud38400 => 802,
            RtuBaud::Baud19200 => 801,
            RtuBaud::Baud9600 => 800,
        };

        self.write_one_word_parameter(0x00, 1, Ok).await?;
        self.write_one_word_parameter(0x02, baud, Ok).await?;
        self.write_one_word_parameter(0x03, 129, Ok).await?;
        self.write_one_word_parameter(0x00, 506, Ok).await?;

        Ok(())
    }

    pub async fn set_target_position_custom(&mut self, value: u32) -> Result<()> {
        const FC: FunctionCode = FunctionCode::Custom(0x78);

        let data = value.to_be_bytes();
        let request = RequestPdu(Request::Custom(FC, &data));

        match self.modbus_transaction(request).await? {
            Response::Custom(FC, d) => {
                if d == data {
                    Ok(())
                } else {
                    Err(Error::UnexpectedResponseData)
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }

    pub async fn set_absolute_position_custom(&mut self, value: u32) -> Result<()> {
        const FC: FunctionCode = FunctionCode::Custom(0x7B);

        let data = value.to_be_bytes();
        let request = RequestPdu(Request::Custom(FC, &data));

        match self.modbus_transaction(request).await? {
            Response::Custom(FC, d) => {
                if d == data {
                    Ok(())
                } else {
                    Err(Error::UnexpectedResponseData)
                }
            }
            _ => Err(Error::UnexpectedResponseType),
        }
    }
}
