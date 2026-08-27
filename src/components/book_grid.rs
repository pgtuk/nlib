use crate::components::BookCard;
use crate::models::Book;
use dioxus::prelude::*;

#[component]
pub fn BookGrid(books: Vec<Book>) -> Element {
    if books.is_empty() {
        return rsx! {
            div { class: "empty-state",
                p { "No books found." }
            }
        };
    }

    rsx! {
        div { class: "book-grid",
            for book in books {
                BookCard { key: "{book.id}", book }
            }
        }
    }
}
