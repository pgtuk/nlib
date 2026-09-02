pub(crate) mod google_books;
pub(crate) mod openlibrary;
pub(crate) mod yakaboo;

use std::sync::LazyLock;
use thiserror::Error;

use crate::models::BookDraft;
pub use google_books::GoogleBooksError;
pub use openlibrary::OpenLibraryError;
pub use yakaboo::YakabooError;

pub(crate) static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default()
});

#[derive(Error, Debug)]
pub enum BookSearchError {
    #[error("{0}")]
    OpenLibrary(#[from] OpenLibraryError),
    #[error("{0}")]
    GoogleBooks(#[from] GoogleBooksError),
    #[error("{0}")]
    Yakaboo(#[from] YakabooError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    Yakaboo,
    OpenLibrary,
    GoogleBooks,
}

impl SearchProvider {
    pub const ALL: &'static [SearchProvider] = &[
        SearchProvider::Yakaboo,
        SearchProvider::OpenLibrary,
        SearchProvider::GoogleBooks,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Yakaboo => "Yakaboo",
            Self::OpenLibrary => "OpenLibrary",
            Self::GoogleBooks => "Google Books",
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<BookDraft>, BookSearchError> {
        match self {
            Self::Yakaboo => yakaboo::search_yakaboo(query)
                .await
                .map_err(BookSearchError::from),
            Self::OpenLibrary => openlibrary::search_openlibrary(query)
                .await
                .map_err(BookSearchError::from),
            Self::GoogleBooks => google_books::search_google_books(query)
                .await
                .map_err(BookSearchError::from),
        }
    }
}
