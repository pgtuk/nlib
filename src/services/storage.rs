use std::path::PathBuf;

use rusqlite::{params, Connection};
use thiserror::Error;

use crate::models::{Book, BookDraft};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

fn get_db_path() -> PathBuf {
    todo!()
}

pub fn init_db() -> Result<Connection> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS  books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            description TEXT,
            cover_url TEXT
        )",
        [],
    )?;

    Ok(conn)
}

// TODO: add pagination
pub fn load_books(conn: &Connection) -> Result<Vec<Book>> {
    let mut stmt = conn
        .prepare("SELECT id, title, author, description, cover_url FROM books ORDER BY id DESC")?;

    let books_iter = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get::<_, i64>(0)? as usize,
            title: row.get(1)?,
            author: row.get(2)?,
            description: row.get(3)?,
            cover_url: row.get(4)?,
        })
    })?;

    let mut books = Vec::new();
    for book in books_iter {
        books.push(book?);
    }
    Ok(books)
}

pub fn save_book(conn: &Connection, book: &BookDraft) -> Result<usize> {
    conn.execute(
        "INSERT INTO books (title, author, description, cover_url) VALUES (?1, ?2, ?3, ?4)",
        params![book.title, book.author, book.description, book.cover_url],
    )?;

    Ok(conn.last_insert_rowid() as usize)
}
