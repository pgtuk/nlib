use dioxus::prelude::*;

mod components;
mod errors;
mod models;
mod services;

use components::{AddBookModal, BookGrid, NavBar};
use models::BookDraft;

use crate::services::storage::Db;

const MAIN_CSS: Asset = asset!("/assets/main.css");

// nice ui/styles here  - https://github.com/pumpbin/pumpbin

fn main() {
    let db = Db::open().expect("Couldn't init storage");
    dioxus::LaunchBuilder::new().with_context(db).launch(App);
}

#[component]
fn App() -> Element {
    // TODO: handle erorrs without panic
    let search_query = use_signal(String::new);
    let mut show_modal = use_signal(|| false);

    let db = use_context::<Db>();

    let mut books = use_signal(|| db.books().unwrap());

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
                    on_save: move |draft: BookDraft| {
                        db.add_book(&draft).unwrap();
                        show_modal.set(false);
                        books.set(db.books().unwrap());
                    },
                }
            }
        }
    }
}
