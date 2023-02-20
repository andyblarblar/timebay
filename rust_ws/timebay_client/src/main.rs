use iced::{Application, Settings};

mod app;
mod error;
mod mqtt;

fn main() {
    simple_log::console("debug").unwrap();

    app::App::run(Settings::default()).unwrap();
}
