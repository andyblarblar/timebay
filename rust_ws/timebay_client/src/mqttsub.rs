//! MQTT subs as an iced::Subscription

use crate::app::{AppMessage, AppState};
use crate::mqtt::MqttClient;
use iced::Subscription;
use std::any::Any;
use std::sync::Arc;
use timebay_common::error::MqttClientError;
use timebay_common::messages::MqttMessage;

type SharedMqtt = Arc<MqttClient>;

/// MQTT connection state machine
enum State {
    /// Initial connection
    Connecting,
    /// Spinning over connections
    Connected(SharedMqtt),
    /// Attempting to reconnect after first connection
    Reconnecting(SharedMqtt),
}

pub fn mqtt_subscription() -> Subscription<AppMessage> {
    struct MqttWorker;

    // This is a state machine that advances between gui updates. It handles the passive communication with mqtt
    iced::subscription::unfold(
        MqttWorker.type_id(),
        State::Connecting,
        |state| async move {
            match state {
                State::Connecting => {
                    // Loop until we connect to broker
                    let cli = loop {
                        let cli = MqttClient::connect("mqtt://localhost:1883").await; //TODO add correct ip

                        if let Err(err) = cli {
                            log::error!("Failed to connect to server with: {}", err);
                            continue;
                        }

                        break cli.unwrap();
                    };

                    let cli = Arc::new(cli);

                    log::info!("Connected to broker!");
                    (
                        Some(AppMessage::StateChange(AppState::Connected {
                            cli: cli.clone(),
                        })),
                        State::Connected(cli),
                    )
                }
                State::Connected(client) => {
                    // Attempt to receive detection
                    let res = client.recv_mqtt_msg().await;

                    match res {
                        Ok(msg) => match msg {
                            MqttMessage::Connection(msg) => {
                                (Some(AppMessage::ConnectNode(msg)), State::Connected(client))
                            }
                            MqttMessage::Disconnection(msg) => (
                                Some(AppMessage::DisconnectNode(msg)),
                                State::Connected(client),
                            ),
                            MqttMessage::Detection(msg) => {
                                (Some(AppMessage::Detection(msg)), State::Connected(client))
                            }
                            _ => (None, State::Connected(client)),
                        },
                        Err(err) => match err {
                            MqttClientError::ConnectionErr(_)
                            | MqttClientError::ExplicitDisconnect => {
                                log::error!("Broker disconnected!");
                                // Transition gui state to disconnect
                                (
                                    Some(AppMessage::StateChange(AppState::Connecting)),
                                    State::Reconnecting(client),
                                )
                            }
                            MqttClientError::SerializationErr(_) => {
                                log::error!("Serialization failed!");
                                (None, State::Connected(client))
                            }
                        },
                    }
                }
                State::Reconnecting(client) => {
                    log::info!("Attempting reconnect...");

                    match client.reconnect().await {
                        Ok(_) => {
                            log::info!("Reconnected to broker!");
                            (
                                // Transition gui back to connected
                                Some(AppMessage::StateChange(AppState::Connected {
                                    cli: client.clone(),
                                })),
                                State::Connected(client),
                            )
                        }
                        Err(err) => {
                            log::error!("Erred with: {} when attempting reconnect!", err);
                            (None, State::Reconnecting(client))
                        }
                    }
                }
            }
        },
    )
}
