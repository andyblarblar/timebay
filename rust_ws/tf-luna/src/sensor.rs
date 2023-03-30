use crate::error::Error;
use crate::types::NineByteCm;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio_serial::{DataBits, Parity, SerialStream, StopBits};

pub struct TfLuna {
    port: SerialStream,
}

impl TfLuna {
    /// Connects to a TFLuna, if possible.
    pub fn new(port: PathBuf) -> Result<Self, Error> {
        let port = SerialStream::open(
            &tokio_serial::new(port.as_os_str().to_str().unwrap(), 115200)
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
