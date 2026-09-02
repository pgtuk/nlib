use thiserror::Error;

use crate::services::book_search::BookSearchError;
use crate::services::covers::CoverError;
use crate::services::library::LibraryError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Library error: {0}")]
    Library(#[from] LibraryError),

    #[error("Search error: {0}")]
    Search(#[from] BookSearchError),

    #[error("Cover error: {0}")]
    Cover(#[from] CoverError),
}
