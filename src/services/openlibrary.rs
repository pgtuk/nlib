use crate::models::BookDraft;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenLibraryResponse {
    pub docs: Vec<OpenLibraryBook>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenLibraryBook {
    pub title: String,
    pub author_name: Option<Vec<String>>,
    pub cover_i: Option<i64>,
    pub first_sentence: Option<FirstSentence>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FirstSentence {
    String(String),
    Object { value: String },
}

pub async fn search_openlibrary(query: &str) -> Result<Vec<BookDraft>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response: OpenLibraryResponse = client
        .get("https://openlibrary.org/search.json")
        .query(&[("q", query), ("limit", "5")])
        .send()
        .await?
        .json()
        .await?;

    let books = response.docs.into_iter().map(doc_to_book).collect();

    Ok(books)
}

fn doc_to_book(doc: OpenLibraryBook) -> BookDraft {
    let author = doc
        .author_name
        .map(|authors| authors.join(", "))
        .unwrap_or_else(|| "Unknown".to_string());

    let cover_url = doc
        .cover_i
        .map(|id| format!("https://covers.openlibrary.org/b/id/{}-M.jpg", id))
        .unwrap_or_default();

    let description = doc
        .first_sentence
        .map(|sentence| match sentence {
            FirstSentence::String(s) => s,
            FirstSentence::Object { value } => value,
        })
        .unwrap_or_default();

    BookDraft {
        title: doc.title,
        author,
        cover_url,
        description,
    }
}
