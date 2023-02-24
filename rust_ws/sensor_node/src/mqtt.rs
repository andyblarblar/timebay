use std::ops::{Deref, DerefMut};

use paho_mqtt::ConnectOptionsBuilder;

use timebay_common::messages::MqttMessage::{Connection, Disconnection};
use timebay_common::messages::{ConnectionMessage, DisconnectionMessage};

use crate::error::Error;

/// Mqtt client abstraction for sensor nodes
pub struct MqttClient {
    cli: timebay_common::mqttclient::MqttClient,
    node_id: u16,
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
    pub async fn connect(node_id: u16, server_id: &str) -> Result<Self, Error> {
        // Topics to sub to
        let subs = ["/zero"];

        // Set LWT
        let mut conn_opt = ConnectOptionsBuilder::default();
        let msg = Disconnection(DisconnectionMessage::new(node_id));
        conn_opt.will_message(msg.try_into()?);

        // Connect to broker
        let cli = timebay_common::mqttclient::MqttClient::connect(
            server_id,
            &format!("node{}", node_id),
            &subs,
            Some(conn_opt.finalize()),
        )
        .await?;

        // Pub connected message
        let msg = Connection(ConnectionMessage::new(node_id));
        cli.publish(msg).await?;

        Ok(Self { cli, node_id })
    }

    pub fn node_id(&self) -> u16 {
        self.node_id
    }
}
