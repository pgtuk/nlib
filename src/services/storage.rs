use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use directories::ProjectDirs;
use rusqlite::{params, Connection};
use thiserror::Error;

use crate::models::{Book, BookDraft};

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

        let conn = Connection::open(db_file_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS  books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                description TEXT,
                cover_url TEXT,
                cover_path TEXT
            )",
            [],
        )?;

        Ok(Db(Arc::new(Mutex::new(conn))))
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.0.lock().expect("SQLite Mutex poisoned")
    }

    pub fn books(&self) -> Result<Vec<Book>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, title, author, description, cover_url, cover_path FROM books ORDER BY id DESC",
        )?;

        let books_iter = stmt.query_map([], |row| {
            Ok(Book {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                description: row.get(3)?,
                cover_url: row.get(4)?,
                cover_path: row.get::<_, Option<String>>(5)?,
            })
        })?;

        let mut books = Vec::new();
        for book in books_iter {
            books.push(book?);
        }
        Ok(books)
    }

    pub fn add_book(&self, draft: &BookDraft) -> Result<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO books (title, author, description, cover_url) VALUES (?1, ?2, ?3, ?4)",
            params![
                draft.title,
                draft.author,
                draft.description,
                draft.cover_url
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn set_cover_path(&self, id: i64, path: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE books SET cover_path = ?1 WHERE id = ?2",
            params![path, id],
        )?;
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
