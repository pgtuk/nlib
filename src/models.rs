use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookLocation {
    #[default]
    Home,
    Borrowed,
    Dacha,
}

impl BookLocation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Borrowed => "borrowed",
            Self::Dacha => "dacha",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Borrowed => "Borrowed",
            Self::Dacha => "Dacha",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Home => "badge-home",
            Self::Borrowed => "badge-borrowed",
            Self::Dacha => "badge-dacha",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "borrowed" => Self::Borrowed,
            "dacha" => Self::Dacha,
            _ => Self::Home,
        }
    }

    pub fn to_index(self) -> u32 {
        match self {
            Self::Home => 0,
            Self::Borrowed => 1,
            Self::Dacha => 2,
        }
    }

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Borrowed,
            2 => Self::Dacha,
            _ => Self::Home,
        }
    }
}

impl std::fmt::Display for BookLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ToSql for BookLocation {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.as_str().into())
    }
}

impl FromSql for BookLocation {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(BookLocation::from_str(s))
    }
}

// for user input
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct BookDraft {
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: String,
    #[serde(skip)]
    pub cover_bytes: Option<Vec<u8>>,
    pub read_by_nika: bool,
    pub read_by_petya: bool,
    #[serde(default)]
    pub location: BookLocation,
}

// for object from db
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_url: String,
    pub read_by_nika: bool,
    pub read_by_petya: bool,
    #[serde(default)]
    pub location: BookLocation,
}
