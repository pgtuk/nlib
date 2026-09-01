// for user input
#[derive(Clone, PartialEq, Debug, Default)]
pub struct BookDraft {
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: String,
    pub cover_bytes: Option<Vec<u8>>,
}

impl BookDraft {}

// for object from db
#[derive(Clone, PartialEq, Debug)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: String,
    pub cover_path: Option<String>,
}
