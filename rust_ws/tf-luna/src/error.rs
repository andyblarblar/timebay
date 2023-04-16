use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Head was not expected value")]
    InvalidHead,
    /// Packet was too short
    #[error("Packet was too short")]
    TooShort,
    /// Error from underlying serial port
    #[error(transparent)]
    SerialErr(#[from] tokio_serial::Error),
    #[error(transparent)]
    IoErr(#[from] std::io::Error),
    #[error("Tf luna port not found")]
    PortNotFound,
    #[error("Checksum failed")]
    ChecksumFailed
}
