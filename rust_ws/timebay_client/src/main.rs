use iced::{Application, Settings};
use log::LevelFilter::Debug;
use simplelog::{ColorChoice, CombinedLogger, TerminalMode};

mod app;
mod error;
mod mqtt;
mod mqttsub;

fn main() {
    CombinedLogger::init(vec![simplelog::TermLogger::new(
        Debug,
        simplelog::ConfigBuilder::default()
            .add_filter_ignore_str("wgpu_core")
            .add_filter_ignore_str("iced_wgpu")
            .add_filter_ignore_str("naga")
            .add_filter_ignore_str("wgpu_hal")
            .add_filter_ignore_str("winit")
            .add_filter_ignore_str("paho_mqtt")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    app::App::run(Settings::default()).unwrap();
}
