use crate::error::Error;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

/// Mqtt client abstraction for client
pub struct MqttClient {
    cli: timebay_common::mqttclient::MqttClient,
}

impl Debug for MqttClient {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

// Deref to client to emulate "inheritance"
impl Deref for MqttClient {
    type Target = timebay_common::mqttclient::MqttClient;

    fn deref(&self) -> &Self::Target {
        &self.cli
    }
}

impl DerefMut for MqttClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cli
    }
}

impl MqttClient {
    pub async fn connect(server_id: &str) -> Result<Self, Error> {
        // Topics to sub to
        let subs = ["/connect", "/disconnect", "/sensors/detection"];

        // Connect to broker
        let cli = timebay_common::mqttclient::MqttClient::connect(server_id, "client", &subs, None)
            .await?;
        Ok(Self { cli })
    }
}
