use std::fs;
use std::path::Path;

use crate::models::{Book, BookDraft};
use crate::services::covers::{self, CoverError};
use crate::services::storage::{Db, StorageError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Book \"{title}\" by {author} already exists in your library")]
    AlreadyExists { title: String, author: String },

    #[error("Database error: {0}")]
    Storage(#[from] StorageError),

    #[error("Cover error: {0}")]
    Cover(#[from] CoverError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Default)]
pub struct ImportSummary {
    pub imported: usize,
    pub skipped_duplicates: usize,
}

#[derive(Clone)]
pub struct LibraryService {
    db: Db,
}

impl LibraryService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn add_book(&self, draft: &BookDraft) -> Result<i64, LibraryError> {
        if self.db.book_exists(&draft.title, &draft.author)? {
            return Err(LibraryError::AlreadyExists {
                title: draft.title.clone(),
                author: draft.author.clone(),
            });
        }

        let id = self.db.add_book(draft)?;
        if let Some(bytes) = &draft.cover_bytes {
            covers::save_cover_bytes(id, bytes)?;
        }
        Ok(id)
    }

    pub fn update_book(&self, book: &Book) -> Result<(), LibraryError> {
        if self
            .db
            .book_exists_other(book.id, &book.title, &book.author)?
        {
            return Err(LibraryError::AlreadyExists {
                title: book.title.clone(),
                author: book.author.clone(),
            });
        }

        self.db.update_book(book)?;
        Ok(())
    }

    pub fn books(&self) -> Result<Vec<Book>, LibraryError> {
        Ok(self.db.books()?)
    }

    pub fn delete_book(&self, id: i64) -> Result<(), LibraryError> {
        self.db.delete_book(id)?;
        covers::delete_cover(id);

        Ok(())
    }

    pub fn export_to_json(&self, path: &Path) -> Result<usize, LibraryError> {
        let books = self.books()?;
        let count = books.len();
        let json = serde_json::to_string_pretty(&books)?;
        fs::write(path, json)?;
        Ok(count)
    }

    pub fn import_from_json(&self, path: &Path) -> Result<ImportSummary, LibraryError> {
        let content = fs::read_to_string(path)?;
        let books: Vec<BookDraft> = serde_json::from_str(&content)?;

        let mut summary = ImportSummary::default();
        for draft in books {
            if self.db.book_exists(&draft.title, &draft.author)? {
                summary.skipped_duplicates += 1;
                continue;
            }

            let _ = self.db.add_book(&draft)?;
            summary.imported += 1;
        }

        Ok(summary)
    }
}
