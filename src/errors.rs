use thiserror::Error;

use crate::services::covers::CoverError;
use crate::services::google_books::GoogleBooksError;
use crate::services::isbndb::IsbnDbError;
use crate::services::library::LibraryError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Library error: {0}")]
    Library(#[from] LibraryError),

    #[error("Google Books error: {0}")]
    GoogleBooks(#[from] GoogleBooksError),

    #[error("ISBNdb error: {0}")]
    IsbnDb(#[from] IsbnDbError),

    #[error("Cover error: {0}")]
    Cover(#[from] CoverError),
}
