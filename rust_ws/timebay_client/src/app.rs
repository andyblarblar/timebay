use crate::mqtt::MqttClient;
use crate::mqttsub::mqtt_subscription;
use crate::splits::Splits;
use derive_more::IsVariant;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, row, Row, Rule, Text};
use iced::{
    executor, Alignment, Application, Command, Element, Length, Renderer, Subscription, Theme,
};
use std::collections::BTreeSet;
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
    /// Current lap we are timing
    lap: Splits,
    /// Last lap
    last_lap: Option<Splits>,
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
                lap: Splits::new(BTreeSet::new()),
                last_lap: None,
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

                // Add node to splits if we haven't started yet
                self.lap.connect_node(id.node_id);
            }
            AppMessage::DisconnectNode(id) => {
                log::info!("Sensor node: {} disconnected", id.node_id);
                if !self.connected_nodes.remove(&id.node_id) {
                    log::error!("Disconnected a non-connected sensor node!");
                }
            }
            AppMessage::Detection(detc) => {
                if self.lap.handle_node_trigger(detc).is_completed() {
                    // Swap current lap to last lap when done
                    self.last_lap = Some(self.lap.clone());
                    self.lap = Splits::new(self.connected_nodes.clone());
                }
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
            AppState::Connected { .. } => {
                let connected_sensors = {
                    container(
                        Row::with_children(
                            self.connected_nodes
                                .iter()
                                .map(|a| Text::new(a.to_string()).into())
                                .collect(),
                        )
                        .spacing(10)
                        .align_items(Alignment::Start),
                    )
                    .height(Length::FillPortion(2))
                    .align_x(Horizontal::Left)
                };

                let body = row![
                    container(
                        column![
                            container(if let Some(ref last) = self.last_lap {
                                last.view(&None)
                            } else {
                                Text::new("Last Run").into()
                            })
                            .height(Length::FillPortion(8)),
                            connected_sensors
                        ]
                        .align_items(Alignment::Center)
                    )
                    .center_x()
                    .center_y()
                    .width(Length::FillPortion(2))
                    .height(Length::Fill),
                    Rule::vertical(50),
                    container(
                        column![self.lap.view(&self.last_lap)].align_items(Alignment::Center)
                    )
                    .center_x()
                    .center_y()
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                ];

                container(body)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        mqtt_subscription()
    }
}
