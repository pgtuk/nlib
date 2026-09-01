use relm4::prelude::*;

mod app;
mod components;
mod errors;
mod models;
mod services;

use app::App;
use services::desktop::{register_icon_theme, setup_desktop_integration};
use services::library::LibraryService;
use services::storage::Db;

fn main() {
    dotenvy::dotenv().ok();
    setup_desktop_integration();

    let db = Db::open().expect("Couldn't init storage");
    let library = LibraryService::new(db);
    let app = RelmApp::new("dev.nlib.app");
    register_icon_theme();
    relm4::set_global_css(include_str!("../assets/style.css"));

    app.run::<App>(library);
}
