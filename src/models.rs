#[derive(Clone, PartialEq, Debug)]
pub struct BookDraft {
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Book {
    pub id: usize,
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: Option<String>,
}
