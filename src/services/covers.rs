use directories::ProjectDirs;
use std::path::PathBuf;
use std::sync::LazyLock;
use thiserror::Error;

static COVERS_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let dir = ProjectDirs::from("", "", "nlib")?.data_dir().join("covers");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir)
});

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

// sync: bytes already in hand (the reuse path)
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
    let bytes = reqwest::get(url).await?.bytes().await?;
    save_cover_bytes(id, &bytes)
}
