use reqwest::Client;
use serde::Deserialize;
use std::sync::LazyLock;
use thiserror::Error;

use crate::models::BookDraft;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("nlib/0.1.0")
        .build()
        .unwrap_or_default()
});

#[derive(Error, Debug)]
pub enum OpenLibraryError {
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Deserialize, Debug)]
struct OpenLibrarySearchResponse {
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Deserialize, Debug)]
struct OpenLibraryDoc {
    title: String,
    author_name: Option<Vec<String>>,
    cover_i: Option<i64>,
    cover_edition_key: Option<String>,
    isbn: Option<Vec<String>>,
    first_sentence: Option<FirstSentence>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum FirstSentence {
    Single(String),
    Multiple(Vec<String>),
    Object { value: String },
}

impl FirstSentence {
    fn into_string(self) -> String {
        match self {
            Self::Single(s) => s,
            Self::Multiple(mut v) => v.drain(..).next().unwrap_or_default(),
            Self::Object { value } => value,
        }
    }
}

pub async fn search_openlibrary(query: &str) -> Result<Vec<BookDraft>, OpenLibraryError> {
    let url = "https://openlibrary.org/search.json";
    let resp = HTTP_CLIENT
        .get(url)
        .query(&[("q", query), ("limit", "5")])
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(OpenLibraryError::Api(format!(
            "OpenLibrary request failed: {}",
            resp.status()
        )));
    }

    let parsed: OpenLibrarySearchResponse = resp.json().await?;

    let books = parsed
        .docs
        .into_iter()
        .map(|doc| {
            let author = doc
                .author_name
                .map(|authors| authors.join(", "))
                .unwrap_or_else(|| "Unknown".to_string());

            let cover_url = if let Some(id) = doc.cover_i {
                format!("https://covers.openlibrary.org/b/id/{id}-L.jpg")
            } else if let Some(olid) = doc.cover_edition_key {
                format!("https://covers.openlibrary.org/b/olid/{olid}-L.jpg")
            } else if let Some(isbn) = doc.isbn.and_then(|list| list.into_iter().next()) {
                format!("https://covers.openlibrary.org/b/isbn/{isbn}-L.jpg")
            } else {
                String::new()
            };

            let description = doc
                .first_sentence
                .map(|s| s.into_string())
                .unwrap_or_default();

            BookDraft::new(doc.title, author)
                .with_description(description)
                .with_cover_url(cover_url)
        })
        .collect();

    Ok(books)
}
