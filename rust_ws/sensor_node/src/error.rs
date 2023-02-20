use crate::dist_sensor::SensorError;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    MqttClientErr(#[from] timebay_common::error::MqttClientError),
    #[error("MQTT subs are not configured properly")]
    WrongSub,
    #[error("Error during serialization")]
    SerializationError(#[from] timebay_common::error::ConversionError),
    #[error(transparent)]
    SensorErr(#[from] SensorError),
    #[error("System clock rolled back during computation")]
    TimeReset(#[from] SystemTimeError),
}
