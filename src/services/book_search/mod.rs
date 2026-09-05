pub(crate) mod google_books;
pub(crate) mod openlibrary;
pub(crate) mod yakaboo;

use std::sync::LazyLock;
use thiserror::Error;

use crate::models::BookDraft;

pub(crate) static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
        .build()
        .unwrap_or_default()
});

#[derive(Error, Debug)]
pub enum BookSearchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Provider(String),
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
            Self::Yakaboo => yakaboo::search_yakaboo(query).await,
            Self::OpenLibrary => openlibrary::search_openlibrary(query).await,
            Self::GoogleBooks => google_books::search_google_books(query).await,
        }
    }
}
