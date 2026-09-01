use directories::ProjectDirs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::LazyLock;
use thiserror::Error;

static COVERS_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let dir = ProjectDirs::from("", "", "nlib")?.data_dir().join("covers");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir)
});

// Maximum dimensions for cached cover files (fits into card width preserving aspect ratio)
const TARGET_WIDTH: u32 = 300;
const TARGET_HEIGHT: u32 = 450;

#[derive(Error, Debug)]
pub enum CoverError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not determine a data directory")]
    NoDataDir,
}

pub fn cover_path(id: i64) -> Option<PathBuf> {
    let mut path = COVERS_DIR.as_ref()?.clone();
    path.push(format!("{id}.jpg"));
    Some(path)
}

pub fn existing_cover(id: i64) -> Option<PathBuf> {
    cover_path(id).filter(|p| p.exists())
}

/// Resizes cover to standard width maintaining aspect ratio and encodes as JPEG
pub fn resize_cover_image(bytes: &[u8]) -> Vec<u8> {
    if let Ok(img) = image::load_from_memory(bytes) {
        let resized = img.resize(
            TARGET_WIDTH,
            TARGET_HEIGHT,
            image::imageops::FilterType::Triangle,
        );
        let mut buffer = Cursor::new(Vec::new());
        if resized
            .write_to(&mut buffer, image::ImageFormat::Jpeg)
            .is_ok()
        {
            return buffer.into_inner();
        }
    }
    bytes.to_vec()
}

/// Asynchronously processes and resizes image bytes off the main thread
pub async fn process_image_bytes(raw: Vec<u8>) -> Vec<u8> {
    relm4::tokio::task::spawn_blocking(move || resize_cover_image(&raw))
        .await
        .unwrap_or_default()
}

/// Fetches cover bytes from URL and resizes them off the main thread
pub async fn fetch_and_resize(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()?;
    let raw = client.get(url).send().await?.bytes().await?;
    Ok(process_image_bytes(raw.to_vec()).await)
}

// sync: bytes already in hand and normalized
pub fn save_cover_bytes(id: i64, bytes: &[u8]) -> Result<PathBuf, CoverError> {
    let path = COVERS_DIR
        .as_ref()
        .ok_or(CoverError::NoDataDir)?
        .join(format!("{id}.jpg"));

    std::fs::write(&path, bytes)?;
    Ok(path)
}

// async: fallback when the preview never loaded
pub async fn download_cover(url: &str, id: i64) -> Result<PathBuf, CoverError> {
    let bytes = fetch_and_resize(url).await?;
    save_cover_bytes(id, &bytes)
}

pub fn delete_cover(id: i64) {
    if let Some(path) = existing_cover(id) {
        let _ = std::fs::remove_file(path);
    }
}
