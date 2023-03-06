mod app;
mod error;
mod mqtt;
mod splits;

use crate::app::App;
use cursive::menu::{Item, Tree};
use cursive::traits::*;
use cursive::views::{LinearLayout, Panel};
use cursive::{
    views::{CircularFocus, Dialog, TextView},
    With as _,
};

fn main() {
    let mut siv = cursive::default();
    siv.set_autohide_menu(false);

    siv.menubar().add_subtree(
        "Actions",
        Tree::new().with(|tree| {
            tree.add_leaf("Zero Sensors", |s| {
                s.add_layer(Dialog::info("Zeroed sensors")) //TODO impl zeroing
            })
        }),
    );

    let app = App::new();

    siv.add_layer(app.view());

    //TODO spawn background thread, need a spsc from ui to thread, then we use cb_sink to change ui from thread

    siv.run();
}
