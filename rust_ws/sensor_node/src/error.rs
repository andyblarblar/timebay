use std::time::SystemTimeError;
use crate::dist_sensor::SensorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MQTT connection error")]
    MqttConnectionFail(#[from] paho_mqtt::Error),
    #[error("MQTT disconnected")]
    MqttDisconnect,
    #[error("MQTT subs are not configured properly")]
    WrongSub,
    #[error("Error during serialization")]
    SerializationError(#[from] timebay_common::error::ConversionError),
    #[error(transparent)]
    SensorErr(#[from] SensorError),
    #[error("System clock rolled back during computation")]
    TimeReset(#[from] SystemTimeError)
}
