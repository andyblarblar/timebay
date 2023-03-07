use crate::app::{App, AppMessage};
use crate::mqttsub::mqtt_subscription;
use crossfire::mpsc::SharedSenderBRecvF;
use crossfire::mpsc::TxBlocking;
use cursive::views::Dialog;
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::runtime::Builder;

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
    broker_host: String,
    gui_tx: cursive::CbSink,
    gui_rx: crossfire::mpsc::RxFuture<AppMessage, SharedSenderBRecvF>,
) {
    // Communications between async and sync code
    let (backend_tx, backend_rx) = crossfire::mpsc::bounded_tx_future_rx_blocking::<AppMessage>(10);

    // Create an async runtime in another thread we can spawn tasks on
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    // Spawn one task that takes mqtt messages and converts them to messages for the app
    let b_tx_mqtt = backend_tx.clone();
    rt.spawn(async move {
        let mut mqtt_stream = Box::pin(mqtt_subscription(broker_host));
        while let Some(msg) = mqtt_stream.next().await {
            b_tx_mqtt.send(msg).await.unwrap();
        }
    });

    // Spawn another that just forwards messages from the gui to the app
    let b_tx_gui = backend_tx.clone();
    rt.spawn(async move {
        while let Ok(msg) = gui_rx.recv().await {
            b_tx_gui.send(msg).await.unwrap();
        }
    });

    loop {
        // Receive messages from the gui or mqtt that were aggregated in the event loop
        let msg = backend_rx.recv().unwrap();

        // Update doesn't have access to gui, so we cheat a bit and do other updates here (update could have this, but it makes the code less portable)
        if msg.is_zero_ack() {
            gui_tx
                .send(Box::new(|s| s.add_layer(Dialog::info("Zeroed sensors"))))
                .unwrap();
            continue;
        }

        // Perform side effects. Make sure to release this lock else we deadlock when the gui updates
        {
            let mut app = app.lock().unwrap();
            let side_effect = app.app.update(msg);

            // Spawn async side effects, they will message us their results
            if let Some(eff) = side_effect {
                let b_tx_eff = backend_tx.clone();
                rt.spawn(async move { b_tx_eff.send(Box::into_pin(eff).await).await.unwrap() });
            }
        }

        // Rerender gui
        gui_tx
            .send(Box::new(move |s| {
                let view = {
                    let app = s.user_data::<Arc<Mutex<SharedState>>>().unwrap();
                    let app = app.lock().unwrap();
                    app.app.view()
                };

                s.pop_layer();//TODO name the view so we only replace it and not dialogs
                s.add_layer(view);
            }))
            .unwrap();
    }
}
