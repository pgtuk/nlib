use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u32)]
pub enum BookLocation {
    #[default]
    Home = 0,
    Borrowed = 1,
    Dacha = 2,
}

struct LocationMeta {
    location: BookLocation,
    as_str: &'static str,
    label: &'static str,
    badge_class: &'static str,
}

const METAS: [LocationMeta; 3] = [
    LocationMeta {
        location: BookLocation::Home,
        as_str: "home",
        label: "Home",
        badge_class: "badge-home",
    },
    LocationMeta {
        location: BookLocation::Borrowed,
        as_str: "borrowed",
        label: "Borrowed",
        badge_class: "badge-borrowed",
    },
    LocationMeta {
        location: BookLocation::Dacha,
        as_str: "dacha",
        label: "Dacha",
        badge_class: "badge-dacha",
    },
];

impl BookLocation {
    pub const LABELS: &'static [&'static str] = &[METAS[0].label, METAS[1].label, METAS[2].label];

    #[inline]
    fn meta(&self) -> &'static LocationMeta {
        &METAS[*self as usize]
    }

    pub fn as_str(&self) -> &'static str {
        self.meta().as_str
    }

    pub fn label(&self) -> &'static str {
        self.meta().label
    }

    pub fn badge_class(&self) -> &'static str {
        self.meta().badge_class
    }

    pub fn to_index(self) -> u32 {
        self as u32
    }

    pub fn from_index(index: u32) -> Self {
        METAS
            .get(index as usize)
            .map(|m| m.location)
            .unwrap_or_default()
    }

    pub fn from_str(s: &str) -> Self {
        METAS
            .iter()
            .find(|m| m.as_str.eq_ignore_ascii_case(s))
            .map(|m| m.location)
            .unwrap_or_default()
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

impl BookDraft {
    pub fn new(title: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: author.into(),
            ..Default::default()
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_cover_url(mut self, cover_url: impl Into<String>) -> Self {
        self.cover_url = cover_url.into();
        self
    }

    pub fn with_cover_bytes(mut self, cover_bytes: Option<Vec<u8>>) -> Self {
        self.cover_bytes = cover_bytes;
        self
    }

    pub fn with_reading_status(mut self, read_by_nika: bool, read_by_petya: bool) -> Self {
        self.read_by_nika = read_by_nika;
        self.read_by_petya = read_by_petya;
        self
    }

    pub fn with_location(mut self, location: BookLocation) -> Self {
        self.location = location;
        self
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_location_properties() {
        assert_eq!(BookLocation::Home.as_str(), "home");
        assert_eq!(BookLocation::Home.label(), "Home");
        assert_eq!(BookLocation::Home.badge_class(), "badge-home");
        assert_eq!(BookLocation::Home.to_index(), 0);

        assert_eq!(BookLocation::Borrowed.as_str(), "borrowed");
        assert_eq!(BookLocation::Borrowed.label(), "Borrowed");
        assert_eq!(BookLocation::Borrowed.badge_class(), "badge-borrowed");
        assert_eq!(BookLocation::Borrowed.to_index(), 1);

        assert_eq!(BookLocation::Dacha.as_str(), "dacha");
        assert_eq!(BookLocation::Dacha.label(), "Dacha");
        assert_eq!(BookLocation::Dacha.badge_class(), "badge-dacha");
        assert_eq!(BookLocation::Dacha.to_index(), 2);
    }

    #[test]
    fn test_book_location_from_index() {
        assert_eq!(BookLocation::from_index(0), BookLocation::Home);
        assert_eq!(BookLocation::from_index(1), BookLocation::Borrowed);
        assert_eq!(BookLocation::from_index(2), BookLocation::Dacha);
        assert_eq!(BookLocation::from_index(99), BookLocation::Home);
    }

    #[test]
    fn test_book_location_from_str() {
        assert_eq!(BookLocation::from_str("home"), BookLocation::Home);
        assert_eq!(BookLocation::from_str("HOME"), BookLocation::Home);
        assert_eq!(BookLocation::from_str("borrowed"), BookLocation::Borrowed);
        assert_eq!(BookLocation::from_str("Borrowed"), BookLocation::Borrowed);
        assert_eq!(BookLocation::from_str("dacha"), BookLocation::Dacha);
        assert_eq!(BookLocation::from_str("unknown"), BookLocation::Home);
    }
}
