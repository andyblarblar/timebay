mod app;
mod backend;
mod error;
mod mqtt;
mod mqttsub;
mod splits;

use crate::app::{App, AppMessage};
use crate::backend::SharedState;
use cursive::menu::Tree;
use cursive::traits::*;
use cursive::views::Dialog;
use flexi_logger::filter::LogLineWriter;
use flexi_logger::{DeferredNow, Logger};
use log::Record;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

fn main() {
    // Broker host can be passed via CLI
    let host: Vec<_> = std::env::args().take(2).collect();

    let mut siv = cursive::default();
    siv.set_autohide_menu(false);

    // Add a way to view logs
    // TODO replace with default logger once https://github.com/gyscos/cursive/pull/719 is merged
    Logger::try_with_env_or_str("trace")
        .expect("Could not create Logger from environment :(")
        .log_to_writer(cursive_flexi_logger_view::cursive_flexi_logger(&siv))
        .filter(Box::new(PhaoMqttFilter {}))
        .format(flexi_logger::colored_with_thread)
        .start()
        .expect("failed to initialize logger!");

    siv.add_global_callback('q', cursive::Cursive::quit);
    siv.add_global_callback(
        '~',
        cursive_flexi_logger_view::toggle_flexi_logger_debug_console,
    );

    // Communications between gui and background thread
    let (backend_tx, backend_rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(10);
    let cb_sink = siv.cb_sink().clone();
    let app = App::new();

    siv.add_layer(app.view());

    // We need to share app between threads using user data, since it must be send
    let shared = Arc::new(Mutex::new(SharedState { app, backend_tx }));
    siv.set_user_data(shared.clone());

    // Setup top menubar, since the app only gives the body
    siv.menubar()
        .add_subtree(
            "Actions",
            Tree::new().with(|tree| {
                tree.add_leaf("Zero Sensors", |s| {
                    // Because backend handles this message, if we aren't connected nothing will happen
                    let tx = s.user_data::<Arc<Mutex<SharedState>>>().unwrap();
                    let tx = tx.lock().unwrap();
                    tx.backend_tx.send(AppMessage::SendZero).unwrap();
                })
            }),
        )
        .add_subtree(
            "Help",
            Tree::new().with(|tree| {
                tree.add_leaf("Controls", |s| {
                    s.add_layer(Dialog::info("Press Q to quit, ~ for debug logs"))
                })
            }),
        );

    // Because we use an Elm like model, this background thread will receive messages from the gui,
    // act on them, and then edit the whole gui (likely a re-render, though not eccentrically)
    std::thread::spawn(move || {
        backend::backend_thread(
            shared,
            host.get(1)
                .unwrap_or(&String::from("localhost"))
                .deref()
                .to_string(),
            cb_sink,
            backend_rx,
        )
    });

    siv.run();
}

struct PhaoMqttFilter {}

impl flexi_logger::filter::LogLineFilter for PhaoMqttFilter {
    fn write(
        &self,
        now: &mut DeferredNow,
        record: &Record,
        log_line_writer: &dyn LogLineWriter,
    ) -> std::io::Result<()> {
        if !record.target().contains("paho") && !record.target().contains("view"){
            log_line_writer.write(now, record)
        } else {
            Ok(())
        }
    }
}
