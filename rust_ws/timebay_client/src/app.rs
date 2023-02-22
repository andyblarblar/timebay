use crate::mqtt::MqttClient;
use derive_more::IsVariant;
use iced::widget::{row, Text};
use iced::{executor, Application, Command, Element, Renderer, Theme, Subscription};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, IsVariant)]
pub enum AppState {
    /// Connecting to MQTT
    Connecting,
    /// Connected to MQTT
    Connected { cli: Arc<Mutex<MqttClient>> },
}

#[derive(Debug)]
pub enum AppMessage {
    /// Transition the system state
    StateChange(AppState),
    /// Sensor node connected
    ConnectNode(u16),
    /// Sensor node disconnected
    DisconnectNode(u16),
}

pub struct App {
    /// Current connection state of the app
    state: AppState,
    /// Connected sensor node ids
    connected_nodes: HashSet<u16>,
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
                connected_nodes: HashSet::new(),
            },
            // Loop until we connect to server
            Command::perform(
                async {
                    let cli = loop {
                        let cli = MqttClient::connect("mqtt://localhost:1883").await;

                        if let Err(err) = cli {
                            log::error!("Failed to connect to server with: {}", err);
                            continue;
                        }

                        break cli.unwrap();
                    };

                    log::info!("Connected to broker!");
                    AppMessage::StateChange(AppState::Connected {
                        cli: Arc::new(Mutex::new(cli)),
                    })
                },
                |f| f,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("timebay")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::StateChange(state) => {
                self.state = state;
            }
            AppMessage::ConnectNode(id) => {
                log::info!("Sensor node: {} connected", id);
                self.connected_nodes.insert(id);
            }
            AppMessage::DisconnectNode(id) => {
                log::info!("Sensor node: {} disconnected", id);
                if !self.connected_nodes.remove(&id) {
                    log::error!("Disconnected a non-connected sensor node!");
                }
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        match &self.state {
            AppState::Connecting => row![Text::new("Connecting...")].into(),
            AppState::Connected { cli } => row![Text::new("HI!")].into(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {

    }
}
