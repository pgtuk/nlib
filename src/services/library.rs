use crate::models::{Book, BookDraft};
use crate::services::covers;
use crate::services::storage::{Db, StorageError};

pub struct LibraryService {
    db: Db,
}

impl LibraryService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn add_book(&self, draft: &BookDraft) -> Result<i64, StorageError> {
        let id = self.db.add_book(draft)?;
        if let Some(bytes) = &draft.cover_bytes {
            let _ = covers::save_cover_bytes(id, bytes);
        }
        Ok(id)
    }

    pub fn books(&self) -> Result<Vec<Book>, StorageError> {
        self.db.books()
    }

    pub fn delete_book(&self, id: i64) -> Result<(), StorageError> {
        self.db.delete_book(id)?;
        covers::delete_cover(id);

        Ok(())
    }
}
