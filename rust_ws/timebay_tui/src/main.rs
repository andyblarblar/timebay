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
use std::sync::{Arc, Mutex};

fn main() {
    let mut siv = cursive::default();
    siv.set_autohide_menu(false);

    //TODO add background screen that has a logger on it

    // Communications between gui and background thread
    let (backend_tx, backend_rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(10);
    let cb_sink = siv.cb_sink().clone();
    let app = App::new();

    siv.add_layer(app.view());

    // We need to share app between threads using user data, since it must be send
    let shared = Arc::new(Mutex::new(SharedState { app, backend_tx }));
    siv.set_user_data(shared.clone());

    // Setup top menubar, since the app only gives the body
    siv.menubar().add_subtree(
        "Actions",
        Tree::new().with(|tree| {
            tree.add_leaf("Zero Sensors", |s| {
                // Because backend handles this message, if we aren't connected nothing will happen
                let tx = s.user_data::<Arc<Mutex<SharedState>>>().unwrap();
                let tx = tx.lock().unwrap();
                tx.backend_tx.send(AppMessage::SendZero).unwrap();
            })
        }),
    );

    // Because we use an Elm like model, this background thread will receive messages from the gui,
    // act on them, and then edit the whole gui (likely a re-render, though not eccentrically)
    std::thread::spawn(move || {
        backend::backend_thread(shared, String::from("localhost"), cb_sink, backend_rx) //TODO get host properly
    });

    siv.run();
}
