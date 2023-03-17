use crate::error::Error;
use crate::types::NineByteCm;
use tokio::io::AsyncReadExt;
use tokio_serial::{DataBits, Parity, SerialPortType, SerialStream, StopBits};

pub struct TfLuna {
    port: SerialStream,
}

impl TfLuna {
    /// Connects to a TFLuna, if possible.
    pub fn new() -> Result<Self, Error> {
        let ports = tokio_serial::available_ports()?;
        let port = ports.iter().find(|p| {
            match &p.port_type {
                SerialPortType::UsbPort(info) => {
                    info.vid == 0x1 && info.pid == 0x123 //TODO make tf luna id
                }
                _ => false,
            }
        });

        let port = port.ok_or(Error::PortNotFound)?;
        let port = SerialStream::open(
            &tokio_serial::new(port.port_name.clone(), 115200)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::One)
                .parity(Parity::None),
        )?;

        Ok(Self { port })
    }

    /// Reads the next reading from the sensor.
    ///
    /// This currently only supports reading the TFLunas default output format.
    pub async fn read(&mut self) -> Result<NineByteCm, Error> {
        // Keep reading until we sync with the header
        while self.port.read_u8().await? != 0x59 {}

        // Read rest of message
        let mut reading = [0u8; 8];
        self.port.read_exact(&mut reading).await?;

        NineByteCm::try_from(reading.as_slice())
    }

    // In the future commands to change the sensors config could be added
}
