#[derive(thiserror::Error, Debug)]
pub enum ConversionError {
    #[error(transparent)]
    PostcardErr(#[from] postcard::Error),
    #[error("This message type cannot actually be sent")]
    NonConvertable,
}

#[derive(thiserror::Error, Debug)]
pub enum MqttClientError {
    /// An error occurred in the mqtt sending or receiving process
    #[error(transparent)]
    ConnectionErr(#[from] paho_mqtt::Error),
    #[error("The broker disconnected")]
    ExplicitDisconnect,
    #[error(transparent)]
    SerializationErr(#[from] ConversionError),
}
