use serde::Deserialize;
use std::sync::LazyLock;
use thiserror::Error;

use crate::models::BookDraft;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("nlib-app/1.0")
        .build()
        .unwrap_or_default()
});

static ISBNDB_API_KEY: LazyLock<Option<String>> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var("ISBNDB_API_KEY")
        .ok()
        .map(|k| k.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|k| !k.is_empty())
});

#[derive(Error, Debug)]
pub enum IsbnDbError {
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("ISBNDB_API_KEY not configured. Add ISBNDB_API_KEY=your_key in .env")]
    MissingApiKey,
    #[error("ISBNdb returned unauthorized (401/403). Check your API key.")]
    Unauthorized,
    #[error("Book not found in ISBNdb")]
    NotFound,
    #[error("ISBNdb error: {0}")]
    Other(String),
}

#[derive(Deserialize, Debug)]
struct IsbnDbBookResponse {
    book: Option<IsbnDbBook>,
}

#[derive(Deserialize, Debug)]
struct IsbnDbSearchResponse {
    books: Option<Vec<IsbnDbBook>>,
}

#[derive(Deserialize, Debug, Clone)]
struct IsbnDbBook {
    title: Option<String>,
    title_long: Option<String>,
    authors: Option<Vec<String>>,
    publisher: Option<String>,
    date_published: Option<String>,
    synopsis: Option<String>,
    overview: Option<String>,
    image: Option<String>,
}

impl From<IsbnDbBook> for BookDraft {
    fn from(b: IsbnDbBook) -> Self {
        let title = b
            .title
            .or(b.title_long)
            .unwrap_or_else(|| "Unknown".to_string());

        let author = b
            .authors
            .filter(|a| !a.is_empty())
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".to_string());

        let mut desc = b.synopsis.or(b.overview).unwrap_or_default();
        if desc.is_empty() {
            if let Some(publ) = b.publisher {
                desc.push_str(&format!("Publisher: {publ}\n"));
            }
            if let Some(date) = b.date_published {
                desc.push_str(&format!("Published: {date}\n"));
            }
        }

        let cover_url = b.image.unwrap_or_default();

        BookDraft {
            title,
            author,
            description: desc.trim().to_string(),
            cover_url,
            cover_bytes: None,
            read_by_nika: false,
            read_by_petya: false,
        }
    }
}

pub async fn search_isbndb(query: &str) -> Result<Vec<BookDraft>, IsbnDbError> {
    let api_key = match ISBNDB_API_KEY.as_ref() {
        Some(key) => key,
        None => return Err(IsbnDbError::MissingApiKey),
    };

    let clean_query: String = query.chars().filter(|c| c.is_alphanumeric()).collect();
    let is_isbn = (clean_query.len() == 10 || clean_query.len() == 13)
        && clean_query
            .chars()
            .all(|c| c.is_ascii_digit() || c == 'X' || c == 'x');

    if is_isbn {
        let url = format!("https://api2.isbndb.com/book/{clean_query}");
        let resp = HTTP_CLIENT
            .get(&url)
            .header("Authorization", api_key)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(IsbnDbError::Unauthorized);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(IsbnDbError::NotFound);
        }
        if !status.is_success() {
            return Err(IsbnDbError::Other(format!("HTTP status {status}")));
        }

        let body: IsbnDbBookResponse = resp.json().await?;
        if let Some(book) = body.book {
            Ok(vec![book.into()])
        } else {
            Err(IsbnDbError::NotFound)
        }
    } else {
        let encoded: String = urlencoding_simple(query.trim());
        let url = format!("https://api2.isbndb.com/books/{encoded}?pageSize=10");
        let resp = HTTP_CLIENT
            .get(&url)
            .header("Authorization", api_key)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(IsbnDbError::Unauthorized);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(IsbnDbError::NotFound);
        }
        if !status.is_success() {
            return Err(IsbnDbError::Other(format!("HTTP status {status}")));
        }

        let body: IsbnDbSearchResponse = resp.json().await?;
        let books = body.books.unwrap_or_default();
        if books.is_empty() {
            return Err(IsbnDbError::NotFound);
        }

        Ok(books.into_iter().map(Into::into).collect())
    }
}

fn urlencoding_simple(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            result.push(b as char);
        } else if b == b' ' {
            result.push('+');
        } else {
            result.push_str(&format!("%{:02X}", b));
        }
    }
    result
}
