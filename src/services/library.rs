use crate::models::{Book, BookDraft};
use crate::services::covers::{self, CoverError};
use crate::services::storage::{Db, StorageError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Database error: {0}")]
    Storage(#[from] StorageError),

    #[error("Cover error: {0}")]
    Cover(#[from] CoverError),
}

pub struct LibraryService {
    db: Db,
}

impl LibraryService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn add_book(&self, draft: &BookDraft) -> Result<i64, LibraryError> {
        let id = self.db.add_book(draft)?;
        if let Some(bytes) = &draft.cover_bytes {
            covers::save_cover_bytes(id, bytes)?;
        }
        Ok(id)
    }

    pub fn books(&self) -> Result<Vec<Book>, LibraryError> {
        Ok(self.db.books()?)
    }

    pub fn delete_book(&self, id: i64) -> Result<(), LibraryError> {
        self.db.delete_book(id)?;
        covers::delete_cover(id);

        Ok(())
    }
}
