pub mod google_books;
pub mod openlibrary;

pub use google_books::{search_google_books, GoogleBooksError};
pub use openlibrary::{search_openlibrary, OpenLibraryError};
use std::sync::LazyLock;

pub(crate) static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()
        .unwrap_or_default()
});
