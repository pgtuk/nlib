pub(crate) mod google_books;
pub(crate) mod openlibrary;

use std::sync::LazyLock;
use thiserror::Error;

use crate::models::BookDraft;
pub use google_books::GoogleBooksError;
pub use openlibrary::OpenLibraryError;

pub(crate) static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()
        .unwrap_or_default()
});

#[derive(Error, Debug)]
pub enum BookSearchError {
    #[error("{0}")]
    OpenLibrary(#[from] OpenLibraryError),
    #[error("{0}")]
    GoogleBooks(#[from] GoogleBooksError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    OpenLibrary,
    GoogleBooks,
}

impl SearchProvider {
    pub const ALL: &'static [SearchProvider] =
        &[SearchProvider::OpenLibrary, SearchProvider::GoogleBooks];

    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenLibrary => "OpenLibrary",
            Self::GoogleBooks => "Google Books",
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<BookDraft>, BookSearchError> {
        match self {
            Self::OpenLibrary => openlibrary::search_openlibrary(query)
                .await
                .map_err(BookSearchError::from),
            Self::GoogleBooks => google_books::search_google_books(query)
                .await
                .map_err(BookSearchError::from),
        }
    }
}
