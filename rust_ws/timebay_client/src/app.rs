use crate::mqtt::MqttClient;
use crate::mqttsub::mqtt_subscription;
use derive_more::IsVariant;
use iced::widget::{row, Button, Text};
use iced::{executor, Application, Command, Element, Renderer, Subscription, Theme};
use std::collections::{BTreeSet};
use std::sync::Arc;
use timebay_common::messages::{
    ConnectionMessage, DetectionMessage, DisconnectionMessage, MqttMessage,
};

#[derive(Debug, IsVariant, Clone)]
pub enum AppState {
    /// Connecting to MQTT
    Connecting,
    /// Connected to MQTT
    Connected { cli: Arc<MqttClient> },
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Transition the system state
    StateChange(AppState),
    /// Sensor node connected
    ConnectNode(ConnectionMessage),
    /// Sensor node disconnected
    DisconnectNode(DisconnectionMessage),
    /// Vehicle detection
    Detection(DetectionMessage),
    /// Zero button was pressed
    SendZero,
    /// Zero op completed
    ZeroAck,
}

pub struct App {
    /// Current connection state of the app
    state: AppState,
    /// Connected sensor node ids
    connected_nodes: BTreeSet<u16>,
    //TODO make run timings struct and view, then mqtt sub
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                state: AppState::Connecting,
                connected_nodes: BTreeSet::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("timebay")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::StateChange(state) => {
                log::debug!("GUI state change to: {:?}", state);
                self.state = state;
            }
            AppMessage::ConnectNode(id) => {
                if self.connected_nodes.insert(id.node_id) {
                    log::info!("Sensor node: {} connected", id.node_id);
                } else {
                    log::debug!("Heartbeat connection from node {}", id.node_id);
                }
            }
            AppMessage::DisconnectNode(id) => {
                log::info!("Sensor node: {} disconnected", id.node_id);
                if !self.connected_nodes.remove(&id.node_id) {
                    log::error!("Disconnected a non-connected sensor node!");
                }
            }
            AppMessage::Detection(detc) => {
                //todo!("handle detection and run state")
            }
            AppMessage::SendZero => match &self.state {
                AppState::Connecting => { /*Cannot zero if not connected*/ }
                AppState::Connected { cli } => {
                    log::info!("Sending zero...");

                    let cli = cli.clone();
                    return Command::perform(
                        async move {
                            if cli.publish(MqttMessage::Zero).await.is_err() {
                                AppMessage::StateChange(AppState::Connecting)
                            } else {
                                AppMessage::ZeroAck
                            }
                        },
                        |f| f,
                    );
                }
            },
            AppMessage::ZeroAck => {
                log::info!("Zero returned success");
                // TODO make a pop up or something
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        match &self.state {
            AppState::Connecting => row![Text::new("Connecting...")].into(),
            AppState::Connected { .. } => row![
                Text::new("HI!"),
                Button::new("a").on_press(AppMessage::SendZero)
            ]
            .into(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        mqtt_subscription()
    }
}
