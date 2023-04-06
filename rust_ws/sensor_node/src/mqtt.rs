use std::ops::{Deref, DerefMut};
use std::time::Duration;

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

        // Set keep alive to be rather long, since otherwise nodes drop too frequently
        conn_opt.keep_alive_interval(Duration::from_millis(10_000));

        // Connect to broker
        let cli = timebay_common::mqttclient::MqttClient::connect(
            server_id,
            &format!("node{}", node_id),
            &subs,
            Some(conn_opt.connect_timeout(Duration::from_secs(2)).finalize()),
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

    /// Reconnects to the broker if disconnected, publishing a connected message if so.
    pub async fn reconnect(&self) {
        loop {
            while let Err(err) = self.cli.reconnect().await {
                log::error!("Erred with {} during reconnect attempt!", err);
            }

            // Pub connection message on reconnect
            if self.pub_connected_msg().await.is_ok() {
                break;
            };
            log::error!("Failed pub after reconnect!");
        }
    }

    /// Convenience method that publishes a connected message for the current node.
    pub async fn pub_connected_msg(&self) -> Result<(), Error> {
        self.cli
            .publish(Connection(ConnectionMessage::new(self.node_id())))
            .await?;
        Ok(())
    }
}
