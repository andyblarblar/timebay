use crate::app::{App, AppMessage};
use crossfire::mpsc::{SharedSenderBRecvF, TxFuture};
use crossfire::mpsc::{SharedSenderFRecvB, TxBlocking};
use cursive::views::Dialog;
use std::sync::{Arc, Mutex};

/// State stored in the gui, can also be accessed in backend
pub struct SharedState {
    /// Application state
    pub app: App,
    /// Channel to send data from the gui to backend
    pub backend_tx: TxBlocking<AppMessage, SharedSenderBRecvF>,
}

/// Backend thread that handles the update and view sections of the program.
///
/// Rx and Tx are used to establish a communications link with the gui.
pub fn backend_thread(
    app: Arc<Mutex<SharedState>>,
    gui_tx: cursive::CbSink,
    gui_rx: crossfire::mpsc::RxFuture<AppMessage, SharedSenderBRecvF>,
) {
    // Comms between async and sync code
    let (backend_tx, backend_rx) = crossfire::mpsc::bounded_tx_future_rx_blocking::<AppMessage>(10);

    std::thread::spawn(|| eventloop(gui_rx, backend_tx));

    loop {
        let msg = backend_rx.recv().unwrap();

        if msg.is_zero_ack() {
            gui_tx
                .send(Box::new(|s| s.add_layer(Dialog::info("Zeroed sensors"))))
                .unwrap();
            continue;
        }

        // Perform side effects
        {
            let mut app = app.lock().unwrap();
            app.app.update(msg);
        }

        gui_tx
            .send(Box::new(move |s| {
                let view = {
                    let app = s.user_data::<Arc<Mutex<SharedState>>>().unwrap();
                    let app = app.lock().unwrap();
                    app.app.view()
                };

                s.pop_layer();
                s.add_layer(view);
            }))
            .unwrap();
    }
}

/// Async event loop to run in a different thread, handling network events and messages from the gui.
fn eventloop(
    gui_rx: crossfire::mpsc::RxFuture<AppMessage, SharedSenderBRecvF>,
    backend_tx: TxFuture<AppMessage, SharedSenderFRecvB>,
) {
    //TODO add tokio, then add MQTT state machine
    // make sure to handle SendZero here, then send ZeroAck to backend
}
