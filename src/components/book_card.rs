use crate::models::Book;
use dioxus::prelude::*;

#[component]
pub fn BookCard(book: Book) -> Element {
    rsx! {
        div { class: "book-card",
            div { class: "book-cover-container",
                if book.cover_url.is_empty() {
                    span { class: "no-cover-placeholder", "No Cover" }
                } else {
                    img { class: "book-cover-img", src: "{book.cover_url}", alt: "{book.title}" }
                }
            }
            div { class: "book-info",
                h3 { class: "book-title", "{book.title}" }
                p { class: "book-author", "by {book.author}" }
                p { class: "book-description", "{book.description}" }
            }
        }
    }
}
