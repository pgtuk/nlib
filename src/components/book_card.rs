use crate::models::Book;
use dioxus::prelude::*;

#[component]
pub fn BookCard(book: Book) -> Element {
    rsx! {
        div { class: "book-card",
            div { class: "book-cover-container",
                if let Some(url) = &book.cover_url {
                    img { class: "book-cover-img", src: "{url}", alt: "{book.title}" }
                } else {
                    span { class: "no-cover-placeholder", "No Cover" }
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
