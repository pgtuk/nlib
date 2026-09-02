mod migrations;

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use directories::ProjectDirs;
use rusqlite::{params, Connection};
use thiserror::Error;

use crate::models::{Book, BookDraft};
use migrations::migrate;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not determine a data directory for this platform")]
    NoDataDir,
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Clone)]
// wraps connection to be able to share it between threads/signals without cloning
pub struct Db(Arc<Mutex<Connection>>);

impl Db {
    pub fn open() -> Result<Self> {
        let db_file_path = get_db_path()?;
        create_dir_all(db_file_path.parent().expect("db path should have parent"))?; // ensure db file path exists

        let mut conn = Connection::open(db_file_path)?;
        migrate(&mut conn)?;

        Ok(Db(Arc::new(Mutex::new(conn))))
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.0.lock().expect("SQLite Mutex poisoned")
    }

    pub fn books(&self) -> Result<Vec<Book>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, title, author, description, cover_url, read_by_nika, read_by_petya, location FROM books ORDER BY id DESC",
        )?;

        let books_iter = stmt.query_map([], |row| {
            Ok(Book {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                description: row.get(3)?,
                cover_url: row.get(4)?,
                read_by_nika: row.get(5)?,
                read_by_petya: row.get(6)?,
                location: row.get(7)?,
            })
        })?;

        let mut books = Vec::new();
        for book in books_iter {
            books.push(book?);
        }
        Ok(books)
    }

    pub fn book_exists(&self, title: &str, author: &str) -> Result<bool> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT 1 FROM books WHERE LOWER(TRIM(title)) = LOWER(TRIM(?1)) AND LOWER(TRIM(author)) = LOWER(TRIM(?2)) LIMIT 1",
        )?;
        let exists = stmt.exists(params![title, author])?;
        Ok(exists)
    }

    pub fn book_exists_other(&self, id: i64, title: &str, author: &str) -> Result<bool> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT 1 FROM books WHERE id != ?1 AND LOWER(TRIM(title)) = LOWER(TRIM(?2)) AND LOWER(TRIM(author)) = LOWER(TRIM(?3)) LIMIT 1",
        )?;
        let exists = stmt.exists(params![id, title, author])?;
        Ok(exists)
    }

    pub fn add_book(&self, draft: &BookDraft) -> Result<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO books (title, author, description, cover_url, read_by_nika, read_by_petya, location) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                draft.title,
                draft.author,
                draft.description,
                draft.cover_url,
                draft.read_by_nika,
                draft.read_by_petya,
                draft.location,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn update_book(&self, book: &Book) -> Result<()> {
        self.conn().execute(
            "UPDATE books SET title = ?1, author = ?2, description = ?3, cover_url = ?4, read_by_nika = ?5, read_by_petya = ?6, location = ?7 WHERE id = ?8",
            params![
                book.title,
                book.author,
                book.description,
                book.cover_url,
                book.read_by_nika,
                book.read_by_petya,
                book.location,
                book.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_book(&self, id: i64) -> Result<()> {
        self.conn()
            .execute("DELETE FROM books WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn get_db_path() -> Result<PathBuf> {
    // should return ~/.local/share/nlib/nlib.sqlite and be distribution agnostic
    Ok(ProjectDirs::from("", "", "nlib")
        .ok_or(StorageError::NoDataDir)?
        .data_dir()
        .join("nlib.sqlite"))
}
