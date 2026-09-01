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
            image::imageops::FilterType::Lanczos3,
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

// sync: bytes already in hand (the reuse path)
pub fn save_cover_bytes(id: i64, bytes: &[u8]) -> Result<PathBuf, CoverError> {
    let path = COVERS_DIR
        .as_ref()
        .ok_or(CoverError::NoDataDir)?
        .join(format!("{id}.jpg"));

    let processed = resize_cover_image(bytes);
    std::fs::write(&path, processed)?;
    Ok(path)
}

// async: fallback when the preview never loaded
pub async fn download_cover(url: &str, id: i64) -> Result<PathBuf, CoverError> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    save_cover_bytes(id, &bytes)
}

pub fn delete_cover(id: i64) {
    if let Some(path) = existing_cover(id) {
        let _ = std::fs::remove_file(path);
    }
}
