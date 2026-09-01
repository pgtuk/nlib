# Nika's Lib (nlib) 📚

A fast, lightweight desktop personal library manager built with **Rust**, **GTK4**, **Libadwaita**, and **Relm4**.

---

## Features

- 📖 **Catalog & Manage Books**: Add, view, edit details, and track reading statuses independently for Nika and Petya.
- 🔍 **Multi-Source Book Search**:
  - **OpenLibrary**: Free community database (searches titles, authors, and ISBNs out-of-the-box).
  - **Google Books**: Rich international metadata with automatic high-resolution cover upscaling (supports title, author, and `isbn:` searches).
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

You can configure an optional Google Books API key by creating a `.env` file in the root directory:

```ini
# Optional: Higher rate limits and quota for Google Books search
GOOGLE_BOOKS_API_KEY="your_google_books_api_key_here"
```

> **Note**: Both **OpenLibrary** and **Google Books** work out-of-the-box without requiring any API key.

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
