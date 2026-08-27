use dioxus::prelude::*;

mod components;
mod errors;
mod models;
mod services;

use components::{AddBookModal, BookGrid, NavBar};
use models::{Book, BookDraft};

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let search_query = use_signal(String::new);
    let mut show_modal = use_signal(|| false);

    let db_conn = services::storage::init_db()?;

    // TODO: handle erorrs without panic
    let mut books = use_signal(|| services::storage::load_books(&db_conn).unwrap());

    let query = search_query.read().to_lowercase();
    let filtered_books = books
        .read()
        .iter()
        .filter(|b| {
            b.title.to_lowercase().contains(&query) || b.author.to_lowercase().contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            class: "app-container",

            NavBar {
                search_query,
                on_add_click: move |_| show_modal.set(true),
            }
            main {
                BookGrid {
                    books: filtered_books
                }
            }
            if show_modal() {
                AddBookModal {
                    on_close: move |_| show_modal.set(false),
                    on_save: move |mut new_book: BookDraft| {

                        show_modal.set(false);
                    },
                }
            }
        }
    }
}
