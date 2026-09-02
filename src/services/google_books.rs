use std::sync::LazyLock;

use crate::models::BookDraft;
use serde::Deserialize;
use thiserror::Error;

static API_KEY: LazyLock<Option<String>> = LazyLock::new(|| {
    std::env::var("GOOGLE_BOOKS_API_KEY")
        .ok()
        .map(|k| {
            k.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string()
        })
        .filter(|k| !k.is_empty())
});

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("nlib/0.1.0")
        .build()
        .unwrap_or_default()
});

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
    extra_large: Option<String>,
    large: Option<String>,
    medium: Option<String>,
    small: Option<String>,
    thumbnail: Option<String>,
    #[serde(rename = "smallThumbnail")]
    small_thumbnail: Option<String>,
}

pub async fn search_google_books(query: &str) -> Result<Vec<BookDraft>, GoogleBooksError> {
    let mut request = HTTP_CLIENT
        .get("https://www.googleapis.com/books/v1/volumes")
        .query(&[("q", query), ("maxResults", "5")]);

    if let Some(key) = API_KEY.as_deref() {
        request = request.query(&[("key", key)]);
    }

    let resp = request.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        if let Ok(err_resp) = serde_json::from_str::<GoogleBooksResponse>(&text) {
            if let Some(err) = err_resp.error {
                if err.message.contains("Quota exceeded") || err.message.contains("RATE_LIMIT") {
                    return Err(GoogleBooksError::Api(
                        "Google Books quota exceeded. Check your API key or use OpenLibrary."
                            .to_string(),
                    ));
                }
                if err.message.contains("API key not valid") || err.message.contains("keyInvalid") {
                    return Err(GoogleBooksError::Api(
                        "Invalid Google Books API key in .env. Please verify your key.".to_string(),
                    ));
                }
                return Err(GoogleBooksError::Api(err.message));
            }
        }
        return Err(GoogleBooksError::Api(format!(
            "Google Books service error ({status})"
        )));
    }

    let response: GoogleBooksResponse = serde_json::from_str(&text).map_err(|e| {
        GoogleBooksError::Api(format!("Failed to parse Google Books response: {e}"))
    })?;

    if let Some(err) = response.error {
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
        .and_then(|img| {
            img.extra_large
                .or(img.large)
                .or(img.medium)
                .or(img.small)
                .or(img.thumbnail)
                .or(img.small_thumbnail)
        })
        .map(|url| {
            // Replace edge=curl and zoom levels for higher resolution images
            let secure = url.replace("http://", "https://");
            secure.replace("edge=curl&", "").replace("zoom=1", "zoom=0")
        })
        .unwrap_or_default();

    BookDraft::new(title, author)
        .with_description(description)
        .with_cover_url(cover_url)
}
