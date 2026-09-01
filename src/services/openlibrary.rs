use serde::Deserialize;

use crate::models::BookDraft;

#[derive(Deserialize, Debug)]
struct OpenLibrarySearchResponse {
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum FirstSentence {
    Text(String),
    Array(Vec<String>),
    Object { value: String },
}

impl FirstSentence {
    fn into_string(self) -> String {
        match self {
            Self::Text(s) => s,
            Self::Array(arr) => arr.into_iter().next().unwrap_or_default(),
            Self::Object { value } => value,
        }
    }
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

pub async fn search_openlibrary(query: &str) -> Result<Vec<BookDraft>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()?;

    let response: OpenLibrarySearchResponse = client
        .get("https://openlibrary.org/search.json")
        .query(&[("q", query), ("limit", "5")])
        .send()
        .await?
        .json()
        .await?;

    let books = response
        .docs
        .into_iter()
        .map(|doc| {
            let author = doc
                .author_name
                .map(|a| a.join(", "))
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

            BookDraft {
                title: doc.title,
                author,
                cover_url,
                description,
                cover_bytes: None,
                read_by_nika: false,
                read_by_petya: false,
            }
        })
        .collect();

    Ok(books)
}
