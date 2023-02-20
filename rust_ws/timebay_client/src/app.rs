use iced::{executor, Application, Command, Element, Renderer, Subscription, Theme};
use std::collections::HashSet;

#[derive(Debug)]
pub enum AppState {
    /// Connecting to MQTT
    Connecting,
    /// Connected to MQTT
    Connected,
}

#[derive(Debug)]
pub enum AppMessage {
    /// Transition the system state
    StateChange(AppState),
    ConnectNode(u16),
    DisconnectNode(u16),
}

pub struct App {
    /// Current connection state of the app
    state: AppState,
    /// Connected sensor node ids
    connected_nodes: HashSet<u16>,
    //TODO make run timings struct
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
            Command::none(), //TODO connect here
        )
    }

    fn title(&self) -> String {
        String::from("timebay")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::StateChange(state) => {
                log::info!("Connected to broker!");
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
        todo!("Make widgets for run timings and then make them here")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        todo!("Make sub that relays mqtt messages")
    }
}
