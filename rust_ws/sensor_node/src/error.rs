use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MQTT connection error")]
    MqttConnectionFail(#[from] paho_mqtt::Error),
    #[error("Error during serialization")]
    SerializationError(#[from] postcard::Error),
}
