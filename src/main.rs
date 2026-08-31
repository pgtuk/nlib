use relm4::prelude::*;

mod app;
mod components;
mod errors;
mod models;
mod services;

use app::App;
use services::storage::Db;

fn main() {
    let db = Db::open().expect("Couldn't init storage");

    let app = RelmApp::new("dev.nlib.app");
    relm4::set_global_css(include_str!("../assets/style.css"));

    app.run::<App>(db);
}
