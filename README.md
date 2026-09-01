# Nika's Lib (nlib) 📚

A fast, lightweight desktop personal library manager built with **Rust**, **GTK4**, **Libadwaita**, and **Relm4**.

---

## Features

- 📖 **Catalog & Manage Books**: Add, view, edit details, and track reading statuses independently for Nika and Petya.
- 🔍 **Multi-Source Book Search**:
  - **OpenLibrary**: Open community registry (no API key required).
  - **Google Books**: Rich international metadata with automatic high-resolution cover upscaling (optional API key for higher rate limits).
  - **ISBNdb API v2**: Fast lookup by exact ISBN-10/13 or keyword search across international and Ukrainian publisher catalogs.
- 🖼️ **High-Resolution Covers**: Local caching with dedicated preview modal cards.
- 🏷️ **Filters & Sorting**:
  - Filter by reading statuses (Read / Unread).
  - Sort by **Date Added**, **Title**, or **Author** (Ascending / Descending).
  - Instant client-side search across titles and authors.
- 💾 **Backup & Restore**:
  - Export your entire library to JSON backup.
  - Import JSON backups with duplicate detection.
- 🎨 **Modern Libadwaita UI**: Responsive grid layout styled with Catppuccin Macchiato colors.

---

## Configuration

You can configure external search provider API keys by creating a `.env` file in the root directory:

```ini
# Optional: Higher rate limits and quota for Google Books search
GOOGLE_BOOKS_API_KEY="your_google_books_api_key_here"

# Optional: Required if using the ISBNdb search provider
ISBNDB_API_KEY="your_isbndb_api_key_here"
```

> **Note**:
> - **OpenLibrary** works out-of-the-box with no API key.
> - **Google Books** functions without an API key for basic searches, but providing `GOOGLE_BOOKS_API_KEY` prevents IP-based rate limiting.
> - **ISBNdb** requires an active API key from [isbndb.com](https://isbndb.com/).

---

## Development & Build

### Requirements
- Rust 1.80+ (or latest stable)
- GTK4 (`libgtk-4-dev`)
- Libadwaita (`libadwaita-1-dev`)

### Run in Debug Mode
```bash
cargo run
```

### Build Binary
```bash
cargo build --release
```
The compiled binary will be placed at `target/release/nlib`.
