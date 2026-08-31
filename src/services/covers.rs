use directories::ProjectDirs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoverError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not determine a data directory")]
    NoDataDir,
}

fn covers_dir() -> Result<PathBuf, CoverError> {
    let dir = ProjectDirs::from("", "", "nlib")
        .ok_or(CoverError::NoDataDir)?
        .data_dir()
        .join("covers");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

// sync: bytes already in hand (the reuse path)
pub fn save_cover_bytes(id: i64, bytes: &[u8]) -> Result<PathBuf, CoverError> {
    let path = covers_dir()?.join(format!("{id}.jpg"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

// async: fallback when the preview never loaded
pub async fn download_cover(url: &str, id: i64) -> Result<PathBuf, CoverError> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    save_cover_bytes(id, &bytes)
}
