use crate::models::BookDraft;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoogleBooksError {
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Api(String),
}

#[derive(Deserialize, Debug)]
struct GoogleBooksResponse {
    items: Option<Vec<GoogleVolumeItem>>,
    error: Option<GoogleApiError>,
}

#[derive(Deserialize, Debug)]
struct GoogleApiError {
    message: String,
}

#[derive(Deserialize, Debug)]
struct GoogleVolumeItem {
    #[serde(rename = "volumeInfo")]
    volume_info: Option<GoogleVolumeInfo>,
}

#[derive(Deserialize, Debug)]
struct GoogleVolumeInfo {
    title: Option<String>,
    authors: Option<Vec<String>>,
    description: Option<String>,
    #[serde(rename = "imageLinks")]
    image_links: Option<GoogleImageLinks>,
}

#[derive(Deserialize, Debug)]
struct GoogleImageLinks {
    thumbnail: Option<String>,
    #[serde(rename = "smallThumbnail")]
    small_thumbnail: Option<String>,
}

pub async fn search_google_books(query: &str) -> Result<Vec<BookDraft>, GoogleBooksError> {
    let client = reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()?;

    let mut request = client
        .get("https://www.googleapis.com/books/v1/volumes")
        .query(&[("q", query), ("maxResults", "5")]);

    if let Ok(key) = std::env::var("GOOGLE_BOOKS_API_KEY") {
        let key = key.trim().to_string();
        if !key.is_empty() {
            request = request.query(&[("key", &key)]);
        }
    }

    let response: GoogleBooksResponse = request.send().await?.json().await?;

    if let Some(err) = response.error {
        if err.message.contains("Quota exceeded") || err.message.contains("RATE_LIMIT") {
            return Err(GoogleBooksError::Api(
                "Google Books quota exceeded. Set the GOOGLE_BOOKS_API_KEY environment variable, or switch to OpenLibrary.".to_string(),
            ));
        }
        return Err(GoogleBooksError::Api(err.message));
    }

    let items = response.items.unwrap_or_default();
    let books = items
        .into_iter()
        .filter_map(|item| item.volume_info)
        .map(volume_to_draft)
        .collect();

    Ok(books)
}

fn volume_to_draft(info: GoogleVolumeInfo) -> BookDraft {
    let title = info.title.unwrap_or_default();
    let author = info
        .authors
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown".to_string());
    let description = info.description.unwrap_or_default();
    let cover_url = info
        .image_links
        .and_then(|img| img.thumbnail.or(img.small_thumbnail))
        .map(|url| url.replace("http://", "https://"))
        .unwrap_or_default();

    BookDraft {
        title,
        author,
        cover_url,
        description,
        cover_bytes: None,
    }
}
