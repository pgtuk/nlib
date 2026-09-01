use crate::services::covers::CoverError;
use crate::services::library::LibraryError;
use crate::services::storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Library(#[from] LibraryError),

    #[error("Database error: {0}")]
    Storage(#[from] StorageError),

    #[error("Cover image error: {0}")]
    Cover(#[from] CoverError),
}
