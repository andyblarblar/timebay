use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    MqttConnectionFail(#[from] timebay_common::error::MqttClientError),
    #[error("MQTT subs are not configured properly")]
    WrongSub,
}
